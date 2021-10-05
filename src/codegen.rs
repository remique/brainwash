use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::{Linkage, Module};
use inkwell::targets::{InitializationConfig, Target};
use inkwell::types::{FunctionType, IntType, PointerType};
use inkwell::values::AnyValue;
use inkwell::values::{BasicValueEnum, IntValue};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;

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

pub struct Codegen<'ctx> {
    pub input: Node,
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
    pub types: Types<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn generate_llvm(&self) {
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

        // Allocate data* and ptr*
        let data_alloca = self
            .builder
            .build_alloca(self.types.i8_ptr_type, "data_alloca");
        let ptr_alloca = self
            .builder
            .build_alloca(self.types.i8_ptr_type, "ptr_alloca");

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
        self.builder.build_store(ptr_alloca, calloc_data_basic_val);

        match &self.input {
            Node::Expr(expr_val) => {
                for x in expr_val.iter() {
                    match x {
                        Node::PlusNode => {
                            let amount_const = self.types.i8_type.const_int(1 as u64, false);
                            let ptr_load = self
                                .builder
                                .build_load(data_alloca, "load_ptr")
                                .into_pointer_value();
                            let ptr_val = self.builder.build_load(ptr_load, "ptr_val");
                            let result = self.builder.build_int_add(
                                ptr_val.into_int_value(),
                                amount_const,
                                "add_data_ptr",
                            );
                            self.builder.build_store(ptr_load, result);
                        }
                        Node::PrintCurrPosNode => {
                            let pc_char = self.builder.build_load(
                                self.builder
                                    .build_load(data_alloca, "ptr_val")
                                    .into_pointer_value(),
                                "load_ptr_val",
                            );

                            let sext = self.builder.build_int_s_extend(
                                pc_char.into_int_value(),
                                self.context.i32_type(),
                                "putchar_sign",
                            );
                            self.builder.build_call(
                                putchar_fn_value,
                                &[sext.into()],
                                "putchar_call",
                            );
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                // error
            }
        }

        // Free the pointer
        self.builder.build_free(
            self.builder
                .build_load(data_alloca, "load")
                .into_pointer_value(),
        );

        let return_zero_const = self.types.i32_type.const_int(0, false);
        self.builder.build_return(Some(&return_zero_const));

        self.module.print_to_stderr();
    }

    fn generate_basic_block() {}
}
