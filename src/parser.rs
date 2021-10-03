use crate::lexer::*;

#[derive(Debug, Clone)]
pub enum Node {
    PlusNode,
    MinusNode,
    IncrementPtrNode,
    DecrementPtrNode,
    PrintCurrPosNode,

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

    pub fn parse_expr(&mut self, vect: &[Token], is_loop: bool) -> Node {
        let mut x: Vec<Node> = Vec::new();

        let mut iq = vect.iter();

        while let Some(tok) = iq.next() {
            match tok {
                Token::PlusToken => {
                    // TODO: check if last item in x is plusnode, if yes then
                    // edit out the plusnode and add 1
                    // then push Node::PlusNode(count+1)
                    x.push(Node::PlusNode);
                    self.position += 1;
                }
                Token::MinusToken => {
                    x.push(Node::MinusNode);
                    self.position += 1;
                }
                Token::SmallerThanToken => {
                    x.push(Node::DecrementPtrNode);
                    self.position += 1;
                }
                Token::GreaterThanToken => {
                    x.push(Node::IncrementPtrNode);
                    self.position += 1;
                }
                Token::DotToken => {
                    x.push(Node::PrintCurrPosNode);
                    self.position += 1;
                }
                Token::LeftBracketToken => {
                    let rest = self.input.clone();
                    self.position += 1;
                    let pos = self.position.clone();

                    // find next ] sign
                    let idx = rest
                        .clone()
                        .iter()
                        .position(|&r| r == Token::RightBracketToken)
                        .unwrap();

                    let asdf = self.parse_expr(&rest[pos..idx], true);

                    x.push(asdf);

                    // skip loop-length times
                    iq.nth(idx - pos);
                }
                Token::RightBracketToken => {
                    self.position += 1;
                }
            }
        }

        if is_loop == true {
            Node::LoopExpr(Box::new(x))
        } else {
            Node::Expr(Box::new(x))
        }
    }

    pub fn parse_all(&mut self) -> Node {
        let x = self.parse_expr(&self.input.clone(), false);
        // println!("{:#?}", x);

        x
    }
}
