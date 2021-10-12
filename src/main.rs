#![allow(unused_imports)]
#![allow(dead_code)]

mod codegen;
mod lexer;
mod parser;

use crate::codegen::*;
use crate::lexer::*;
use crate::parser::*;

use clap::{App, AppSettings, Arg, SubCommand};
use inkwell::context::Context;

fn main() {
    let matches = App::new("Brainwash")
        .setting(AppSettings::DisableVersion)
        .author("Emil Jaszczuk <emj1054@gmail.com>")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to compile")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("profiler")
                .short("p")
                .long("profiler")
                .multiple(false)
                .help("Shows how long each step takes"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Sets the output file")
                .default_value("bfo")
                .takes_value(true),
        )
        .get_matches();

    let l = Lexer::new(matches.value_of("INPUT").unwrap());
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
