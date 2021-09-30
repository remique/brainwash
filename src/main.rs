#![allow(unused_imports)]
#![allow(dead_code)]

mod lexer;
mod parser;

use crate::lexer::*;
use crate::parser::*;

use std::iter::{Iterator, Peekable};
fn main() {
    let l = Lexer::new("test.bf");
    l.check_loops().unwrap();

    let mut p = Parser::new(l.tokens);
    p.parse_all();
}
