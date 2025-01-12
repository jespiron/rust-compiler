// Lexer: charstream -> tokens

use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug)]
pub enum Token {}

pub fn tokenize(file: File) -> Vec<Token> {
    let tokens = vec![];

    let mut reader = BufReader::new(file);
    let mut buffer = [0; 1];
    while let Ok(1) = reader.read(&mut buffer) {
        let c = buffer[0] as char;

        // Use a match statement to handle each character
        match c {
            'a'..='z' => println!("Lowercase letter: {}", c),
            'A'..='Z' => println!("Uppercase letter: {}", c),
            '0'..='9' => println!("Digit: {}", c),
            ' ' | '\t' => println!("Whitespace: {:?}", c),
            '\n' => println!("Newline"),
            _ => println!("Other character: {}", c),
        }
    }
    tokens
}
