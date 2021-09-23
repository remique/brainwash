#![allow(unused_imports)]
#![allow(dead_code)]

mod lexer;

use crate::lexer::Lexer;

fn main() {
    let l = Lexer::new("test.bf");
    l.check_loops().unwrap();
}
