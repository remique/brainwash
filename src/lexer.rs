use std::error::Error;
use std::{fmt, fs, fs::File, io::Write, path::Path};

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    pub fn new<T: AsRef<Path>>(filename: T) -> Result<Lexer, std::io::Error> {
        let chars_vec = fs::read(filename)?
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

        Ok(Lexer { tokens })
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

    // This is used for tests only, so not care about cloning
    fn get_tokens(&self) -> Vec<Token> {
        self.tokens.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    static FOO_FILE: &str = "foo_test.bf";
    static LEX_TOKENS_FILE: &str = "lex_test.bf";
    static VALID_BRACKETS_FILE: &str = "brackets_valid_test.bf";
    static INVALID_BRACKETS_FILE: &str = "brackets_invalid_test.bf";

    #[test]
    fn test_valid_filename() {
        let mut file = File::create(FOO_FILE).unwrap();
        file.write_all(b"++++.").unwrap();

        let lx = Lexer::new(FOO_FILE);
        assert!(lx.is_ok());

        fs::remove_file(FOO_FILE).unwrap();
    }

    #[test]
    fn test_invalid_filename() {
        let lx = Lexer::new("does_not_exist.bf");
        assert!(lx.is_err());
    }

    #[test]
    fn test_lex_tokens() {
        let mut file = File::create(LEX_TOKENS_FILE).unwrap();
        file.write_all(b"+-[+]><.").unwrap();

        let lx = Lexer::new(LEX_TOKENS_FILE).unwrap();
        let test_tokens: Vec<Token> = vec![
            Token::PlusToken,
            Token::MinusToken,
            Token::LeftBracketToken,
            Token::PlusToken,
            Token::RightBracketToken,
            Token::GreaterThanToken,
            Token::SmallerThanToken,
            Token::DotToken,
        ];

        assert_eq!(lx.tokens, test_tokens);

        fs::remove_file(LEX_TOKENS_FILE).unwrap();
    }

    #[test]
    fn test_valid_brackets() {
        let mut file = File::create(VALID_BRACKETS_FILE).unwrap();
        file.write_all(b"[[]+[]][]").unwrap();

        let lx = Lexer::new(VALID_BRACKETS_FILE).unwrap();

        assert!(lx.check_loops().is_ok());

        fs::remove_file(VALID_BRACKETS_FILE).unwrap();
    }

    #[test]
    fn test_invalid_brackets() {
        let mut file = File::create(INVALID_BRACKETS_FILE).unwrap();
        file.write_all(b"[[]").unwrap();

        let lx = Lexer::new(INVALID_BRACKETS_FILE).unwrap();

        assert!(lx.check_loops().is_err());

        fs::remove_file(INVALID_BRACKETS_FILE).unwrap();
    }
}
