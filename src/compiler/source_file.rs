use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Bytes, Read};
use std::iter;

use compiler::SourceIter;
use parser::Lexer;
use parser::Token;

/// A struct which represents an on disk assembly file
pub struct SourceFile<'a> {

    /// An optional parent file which included this file
    pub parent: Option<&'a SourceFile<'a>>,

    /// A ID used for debugging purposes
    pub id: i32,

    /// The path to the file without the filename
    pub path: String,

    /// The file's filename without the leading path
    pub filename: String,

    bytes: iter::Peekable<Bytes<File>>,
    last: u8,
    empty: bool
}

impl <'a>SourceFile<'a> {

    pub fn new(parent: Option<&'a SourceFile<'a>>, path: PathBuf) -> Result<SourceFile<'a>, String> {

        let filepath = path.to_str().unwrap();
        match File::open(filepath) {
            Ok(file) => Ok(SourceFile {
                parent: parent,
                id: 0,
                path: path.parent().unwrap_or(Path::new("")).to_str().unwrap().to_string(),
                filename: path.file_name().unwrap().to_str().unwrap().to_string(),
                bytes: file.bytes().peekable(),
                last: 0,
                empty: false
            }),
            Err(err) => Err(format!("Failed to open file \"{}\": {}", filepath, err))
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

impl <'a>SourceIter for SourceFile<'a> {

    fn get(&self) -> u8 {
        self.last
    }

    fn next(&mut self) -> u8 {
        self.last = match self.bytes.next() {
            Some(o) => o.unwrap_or(0),
            None => {
                self.empty = true;
                0
            }
        };
        self.last
    }

    fn peek(&mut self) -> u8 {
        match self.bytes.peek() {
            Some(o) => match o {
                &Ok(n) => n,
                &Err(_) => {
                    self.empty = true;
                    0
                }
            },
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

