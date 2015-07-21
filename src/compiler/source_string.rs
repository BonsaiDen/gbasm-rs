use compiler::SourceIter;
use std::iter;
use std::str;

use parser::Lexer;
use parser::Token;

/// A struct which represents an assembly source from a string
pub struct SourceString<'a> {
    path: &'a str,
    bytes: iter::Peekable<str::Bytes<'a>>,
    last: u8,
    empty: bool
}

impl <'a>SourceString<'a> {

    pub fn new(path: &'a str, source: &'a str) -> SourceString<'a> {
        SourceString {
            path: path,
            bytes: source.bytes().peekable(),
            last: 0,
            empty: false
        }
    }

    pub fn parse(&mut self) {

        let mut lexer = Lexer::new(self).peekable();

        loop {
            match lexer.next().unwrap() {
                Token::Eof => {
                    break;
                },
                Token::Error(ref err) => {
                    println!("Error: {}", err);
                    break;
                },
                token => println!("{:?}", token)
            }
        }

    }

}

impl <'a>SourceIter for SourceString<'a> {

    fn get(&self) -> u8 {
        self.last
    }

    fn next(&mut self) -> u8 {
        self.last = match self.bytes.next() {
            Some(o) => o,
            None => {
                self.empty = true;
                0
            }
        };
        self.last
    }

    fn peek(&mut self) -> u8 {
        match self.bytes.peek() {
            Some(o) => *o,
            None => {
                self.empty = true;
                0
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.empty
    }

}

