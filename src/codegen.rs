use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::{Linkage, Module};
use inkwell::passes::PassManager;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use inkwell::types::{FunctionType, IntType, PointerType};
use inkwell::values::{AnyValue, FunctionValue, PointerValue};
use inkwell::values::{BasicValueEnum, IntValue};
use inkwell::AddressSpace;
use inkwell::IntPredicate;
use inkwell::OptimizationLevel;

use std::fmt::Display;
use std::path::Path;

use crate::parser::Node;

pub struct Types<'ctx> {
    i32_type: IntType<'ctx>,
    i64_type: IntType<'ctx>,
    i8_type: IntType<'ctx>,
    i8_ptr_type: PointerType<'ctx>,
    main_fn_type: FunctionType<'ctx>,
    calloc_fn_type: FunctionType<'ctx>,
    putchar_fn_type: FunctionType<'ctx>,
}

impl<'ctx> Types<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let i32_type = context.i32_type();
        let i64_type = context.i64_type();
        let i8_type = context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(AddressSpace::Generic);
        let main_fn_type = i32_type.fn_type(&[], false);
        let calloc_fn_type = i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let putchar_fn_type = i32_type.fn_type(&[i32_type.into()], false);

        Types {
            i32_type,
            i64_type,
            i8_type,
            i8_ptr_type,
            main_fn_type,
            calloc_fn_type,
            putchar_fn_type,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Loop<'ctx> {
    check: BasicBlock<'ctx>,
    if_true: BasicBlock<'ctx>,
    if_false: BasicBlock<'ctx>,
}

pub struct Codegen<'ctx> {
    pub input: Node,
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
    pub types: Types<'ctx>,
    pub passes: PassManager<FunctionValue<'ctx>>,
    // pub loop_stack: Vec<Loop<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn generate_llvm<T: AsRef<Path> + Display>(&self, filename: T) {
        let loop_stack: Vec<Loop> = Vec::new();

        // Values
        let main_fn_value = self
            .module
            .add_function("main", self.types.main_fn_type, None);

        let calloc_fn_value =
            self.module
                .add_function("calloc", self.types.calloc_fn_type, Some(Linkage::External));

        let putchar_fn_value = self.module.add_function(
            "putchar",
            self.types.putchar_fn_type,
            Some(Linkage::External),
        );

        let entry_block = self.context.append_basic_block(main_fn_value, "entry");
        self.builder.position_at_end(entry_block);

        // Allocate data*
        let data_alloca = self
            .builder
            .build_alloca(self.types.i8_ptr_type, "data_alloca");

        let mem_size_const = self.types.i64_type.const_int(1024, false);
        let element_size_const = self.types.i64_type.const_int(1, false);

        let calloc_data = self.builder.build_call(
            calloc_fn_value,
            &[mem_size_const.into(), element_size_const.into()],
            "calloc_call",
        );

        let calloc_data_result: Result<_, _> = calloc_data.try_as_basic_value().flip().into();
        let calloc_data_basic_val = calloc_data_result
            .map_err(|_| "calloc returned void")
            .unwrap();

        self.builder.build_store(data_alloca, calloc_data_basic_val);

        if let Node::Expr(expr_val) = &self.input {
            self.match_input(
                expr_val,
                data_alloca,
                putchar_fn_value,
                main_fn_value,
                loop_stack,
            );
        }

        self.builder
            .build_return(Some(&self.types.i32_type.const_int(0, false)));

        self.passes.run_on(&main_fn_value);

        self.module
            .print_to_file(format!("./{}.ll", filename))
            .unwrap();
    }

    fn match_input(
        &self,
        input: &Vec<Node>,
        data_alloca: PointerValue<'ctx>,
        putchar_fn_value: FunctionValue,
        main_fn_value: FunctionValue,
        mut loop_stack: Vec<Loop<'ctx>>,
    ) {
        for node_type in input.iter() {
            match node_type {
                Node::PlusNode => self.emit_change_data_value(data_alloca, 1),
                Node::MinusNode => self.emit_change_data_value(data_alloca, -1),
                Node::IncrementPtrNode => self.emit_move_pointer(data_alloca, 1),
                Node::DecrementPtrNode => self.emit_move_pointer(data_alloca, -1),
                Node::PrintCurrPosNode => self.emit_putchar(data_alloca, putchar_fn_value),
                Node::LoopExpr(expr_val) => {
                    self.build_start_loop(&mut loop_stack, data_alloca, main_fn_value);
                    self.match_input(
                        expr_val,
                        data_alloca,
                        putchar_fn_value,
                        main_fn_value,
                        loop_stack.clone(),
                    );
                }
                Node::LoopCloseNode => self.build_loop_close(&mut loop_stack),
                _ => {}
            }
        }
    }

    fn build_start_loop(
        &self,
        loop_stack: &mut Vec<Loop<'ctx>>,
        data_ptr: PointerValue<'ctx>,
        main_fn_value: FunctionValue,
    ) {
        let new_loop = Loop {
            check: self.context.append_basic_block(main_fn_value, "check_loop"),
            if_true: self
                .context
                .append_basic_block(main_fn_value, "if_true_loop"),
            if_false: self
                .context
                .append_basic_block(main_fn_value, "if_false_loop"),
        };

        loop_stack.push(new_loop);
        let last_one = loop_stack.last().unwrap();

        self.builder.build_unconditional_branch(last_one.check);
        self.builder.position_at_end(last_one.check);

        let zero = self.types.i8_type.const_int(0, false);
        let value = self.load_current_value(data_ptr);

        let compare = self
            .builder
            .build_int_compare(IntPredicate::NE, value, zero, "compare");

        self.builder
            .build_conditional_branch(compare, last_one.if_true, last_one.if_false);

        self.builder.position_at_end(last_one.if_true);
    }

    fn build_loop_close(&self, loop_stack: &mut Vec<Loop<'ctx>>) {
        if let Some(block) = loop_stack.pop() {
            self.builder.build_unconditional_branch(block.check);
            self.builder.position_at_end(block.if_false);
        }
    }

    fn emit_change_data_value(&self, data_ptr: PointerValue<'ctx>, value: i32) {
        let amount_const = self.types.i8_type.const_int(value as u64, false);

        let pointer = self.load_current_pointer(data_ptr);
        let value = self.load_current_value(data_ptr);

        let result = self
            .builder
            .build_int_add(value, amount_const, "add_data_ptr");

        self.builder.build_store(pointer, result);
    }

    fn emit_move_pointer(&self, data_ptr: PointerValue<'ctx>, value: i32) {
        let amount_const = self.types.i32_type.const_int(value as u64, false);

        let pointer = self.load_current_pointer(data_ptr);

        let result = unsafe {
            self.builder
                .build_in_bounds_gep(pointer, &[amount_const], "move_ptr")
        };

        self.builder.build_store(data_ptr, result);
    }

    fn emit_putchar(&self, data_ptr: PointerValue, putchar_callee: FunctionValue) {
        let pc_char = self.builder.build_load(
            self.builder
                .build_load(data_ptr, "load_ptr")
                .into_pointer_value(),
            "load_ptr_val",
        );

        let sext = self.builder.build_int_s_extend(
            pc_char.into_int_value(),
            self.context.i32_type(),
            "putchar_sign",
        );
        self.builder
            .build_call(putchar_callee, &[sext.into()], "putchar_call");
    }

    /// Helper function that loads the value stored at current pointer
    fn load_current_value(&self, data_ptr: PointerValue<'ctx>) -> IntValue<'ctx> {
        let ptr_load = self
            .builder
            .build_load(data_ptr, "load_ptr")
            .into_pointer_value();

        self.builder
            .build_load(ptr_load, "ptr_val")
            .into_int_value()
    }

    fn load_current_pointer(&self, data_ptr: PointerValue<'ctx>) -> PointerValue<'ctx> {
        self.builder
            .build_load(data_ptr, "load_ptr")
            .into_pointer_value()
    }
}
