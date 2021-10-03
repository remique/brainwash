#![allow(unused_imports)]
#![allow(dead_code)]

mod lexer;
mod parser;

use crate::lexer::*;
use crate::parser::*;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::{Linkage, Module};
use inkwell::targets::{InitializationConfig, Target};
use inkwell::values::{BasicValueEnum, IntValue};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;

// moze struct typu Types gdzie bede mial wszystkie
// array types itp itd a potem to wjebac jako obiekt do Codegen

struct Codegen<'ctx> {
    input: Node,
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    fn generate_llvm(&self) {
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let array_type = i32_type.array_type(3);

        let fn_value = self.module.add_function("main", fn_type, None);
        let entry = self.context.append_basic_block(fn_value, "entry");

        self.builder.position_at_end(entry);

        let array_alloca = self.builder.build_alloca(array_type, "arr");
        let array = self
            .builder
            .build_load(array_alloca, "arr_l")
            .into_array_value();

        let const_int1 = i32_type.const_int(2, false);
        let const_int2 = i32_type.const_int(3, false);
        let const_int3 = i32_type.const_int(4, false);
        self.builder
            .build_insert_value(array, const_int1, 0, "insert");
        self.builder
            .build_insert_value(array, const_int2, 1, "insert");
        self.builder
            .build_insert_value(array, const_int3, 2, "insert");

        match &self.input {
            Node::Expr(expr_val) => {
                for x in expr_val.iter() {
                    // println!("{:?}", x);
                    match x {
                        Node::PlusNode => {
                            // tutaj generujemy
                        }
                        Node::PrintCurrPosNode => {
                            // bierzemy current value pointer
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                // error
            }
        }

        let pointee = i32_type.const_int(0, false);
        self.builder.build_return(Some(&pointee));

        self.module.print_to_stderr();
    }

    fn generate_basic_block() {}
}

fn main() {
    let l = Lexer::new("test.bf");
    l.check_loops().unwrap();

    let mut p = Parser::new(l.tokens);
    let nodes = p.parse_all();

    let context = Context::create();
    let module = context.create_module("bfc");
    let builder = context.create_builder();
    let execution_engine = module.create_execution_engine().unwrap();

    let cdg = Codegen {
        input: nodes,
        context: &context,
        module,
        builder,
        execution_engine,
    };

    cdg.generate_llvm();
}
