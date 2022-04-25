use crate::lexer::*;

use std::{fmt, fs, fs::File, io::Write, path::Path};

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    PlusNode,
    MinusNode,
    IncrementPtrNode,
    DecrementPtrNode,
    PrintCurrPosNode,
    LoopCloseNode,

    Expr(Box<Vec<Node>>),
    LoopExpr(Box<Vec<Node>>),
}

pub struct Parser {
    pub input: Vec<Token>,
    pub position: usize,
}

impl Parser {
    pub fn new(input: Vec<Token>) -> Self {
        Self { input, position: 0 }
    }

    fn parse_expr(&mut self, vect: &[Token], is_loop: bool) -> Node {
        let mut result: Vec<Node> = Vec::new();

        let mut vect_iter = vect.iter();
        while let Some(tok) = vect_iter.next() {
            match tok {
                Token::PlusToken => {
                    result.push(Node::PlusNode);
                }
                Token::MinusToken => {
                    result.push(Node::MinusNode);
                }
                Token::SmallerThanToken => {
                    result.push(Node::DecrementPtrNode);
                }
                Token::GreaterThanToken => {
                    result.push(Node::IncrementPtrNode);
                }
                Token::DotToken => {
                    result.push(Node::PrintCurrPosNode);
                }
                Token::LeftBracketToken => {
                    let rest = self.input.clone();
                    let pos = self.position.clone() + 1;

                    // find next ] sign with an offset
                    let (_, right) = rest.split_at(pos);
                    let idx = right
                        .clone()
                        .iter()
                        .position(|&r| r == Token::RightBracketToken)
                        .unwrap();

                    let rest = self.input.clone();
                    let real_idx = idx + pos;

                    let new_parsed = self.parse_expr(&rest[pos..real_idx], true);
                    result.push(new_parsed);

                    // skip loop-length times
                    vect_iter.nth(real_idx - pos - 1);
                }
                Token::RightBracketToken => {
                    result.push(Node::LoopCloseNode);
                }
            }
            self.position += 1;
        }

        if is_loop == true {
            Node::LoopExpr(Box::new(result))
        } else {
            Node::Expr(Box::new(result))
        }
    }

    pub fn parse_all(&mut self) -> Node {
        self.parse_expr(&self.input.clone(), false)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    static SIMPLE_NOLOOP_FILE: &str = "simple_noloop_test.bf";
    static SIMPLE_LOOP_FILE: &str = "simple_loop_test.bf";

    #[test]
    fn test_valid_simple_noloops() {
        let mut file = File::create(SIMPLE_NOLOOP_FILE).unwrap();
        file.write_all(b"+.-").unwrap();

        let lx = Lexer::new(SIMPLE_NOLOOP_FILE).unwrap();
        let mut px = Parser::new(lx.tokens);

        let res = Node::Expr(Box::new(vec![
            Node::PlusNode,
            Node::PrintCurrPosNode,
            Node::MinusNode,
        ]));

        assert_eq!(px.parse_all(), res);
        fs::remove_file(SIMPLE_NOLOOP_FILE).unwrap();
    }

    #[test]
    fn test_valid_simple_loop() {
        let mut file = File::create(SIMPLE_LOOP_FILE).unwrap();
        file.write_all(b"+.[-]+").unwrap();

        let lx = Lexer::new(SIMPLE_LOOP_FILE).unwrap();
        let mut px = Parser::new(lx.tokens);

        let res = Node::Expr(Box::new(vec![
            Node::PlusNode,
            Node::PrintCurrPosNode,
            Node::LoopExpr(Box::new(vec![Node::MinusNode])),
            Node::LoopCloseNode,
            Node::PlusNode,
        ]));

        assert_eq!(px.parse_all(), res);
        fs::remove_file(SIMPLE_LOOP_FILE).unwrap();
    }
}
