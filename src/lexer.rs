use std::error::Error;
use std::fmt;
use std::fs;

#[derive(Debug)]
pub enum LexerError {
    BracketsInvalidError,
}

impl std::error::Error for LexerError {}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexerError::BracketsInvalidError => write!(f, "Loop brackets are missing"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    PlusToken,
    MinusToken,
    LeftBracketToken,
    RightBracketToken,
    GreaterThanToken,
    SmallerThanToken,
    DotToken,
}

pub struct Lexer {
    pub tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(filename: &str) -> Lexer {
        let chars_vec = fs::read(filename.to_string())
            .unwrap()
            .iter()
            .filter(|x| **x != 10 as u8)
            .map(|x| *x)
            .collect::<Vec<_>>();

        let mut tokens: Vec<Token> = Vec::new();

        for ch in chars_vec {
            match ch as char {
                '+' => tokens.push(Token::PlusToken),
                '-' => tokens.push(Token::MinusToken),
                '.' => tokens.push(Token::DotToken),
                '[' => tokens.push(Token::LeftBracketToken),
                ']' => tokens.push(Token::RightBracketToken),
                '<' => tokens.push(Token::SmallerThanToken),
                '>' => tokens.push(Token::GreaterThanToken),
                _ => {}
            }
        }

        Lexer { tokens }
    }

    pub fn check_loops(&self) -> Result<(), LexerError> {
        let mut stack = Vec::new();

        for tok in &self.tokens {
            match tok {
                Token::LeftBracketToken => stack.push(Token::LeftBracketToken),
                Token::RightBracketToken => {
                    if stack.pop() != Some(Token::LeftBracketToken) {
                        return Err(LexerError::BracketsInvalidError);
                    }
                }
                _ => {}
            }
        }

        if !stack.is_empty() {
            return Err(LexerError::BracketsInvalidError);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_filename() {
        // create file and test that it exists
        unimplemented!();
    }

    #[test]
    fn test_invalid_filename() {
        unimplemented!();
    }

    #[test]
    fn test_lex() {
        unimplemented!();
    }

    #[test]
    fn test_omit_chars() {
        unimplemented!();
    }

    #[test]
    fn test_valid_brackets() {
        unimplemented!();
    }

    #[test]
    fn test_invalid_brackets() {
        unimplemented!();
    }
}
