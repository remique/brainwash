#![allow(unused_imports)]
#![allow(dead_code)]

mod codegen;
mod lexer;
mod parser;

use crate::codegen::*;
use crate::lexer::*;
use crate::parser::*;

use inkwell::context::Context;

fn main() {
    let l = Lexer::new("test.bf");
    l.check_loops().unwrap();

    let mut p = Parser::new(l.tokens);
    let nodes = p.parse_all();

    let context = Context::create();
    let module = context.create_module("bfc");
    let builder = context.create_builder();
    let execution_engine = module.create_execution_engine().unwrap();

    let types = Types::new(&context);

    let cdg = Codegen {
        input: nodes,
        context: &context,
        module,
        builder,
        execution_engine,
        types,
    };

    cdg.generate_llvm();
}
