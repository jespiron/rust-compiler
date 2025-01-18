use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Literals
    Identifier(String),
    StringLiteral(String),
    Number(f64),

    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Dot,
    Comma,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,

    // One or two character tokens
    Less,
    LessEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Bang,
    BangEqual,

    // Reserved Keywords
    Const,
    Void,
    Int,
    Char,
    Double,
    Struct,
    If,
    Else,
    Switch,
    Case,
    Default,
    While,
    For,
    Do,
    Return,
    Break,
    Continue,
    Print,
    Scan,

    // EOF
    Eof,
}

pub fn tokenize(file: File) -> Vec<Token> {
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader
        .read_to_string(&mut contents)
        .expect("Failed to read file");
    tokenize_from_string(&contents)
}

pub fn tokenize_from_string(contents: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut chars = contents.chars().peekable();
    let mut current = String::new();
    let mut in_string = false;

    while let Some(c) = chars.next() {
        if in_string {
            if c == '"' {
                tokens.push(Token::StringLiteral(current.clone()));
                current.clear();
                in_string = false;
            } else {
                current.push(c);
            }
            continue;
        }

        match c {
            'a'..='z' | 'A'..='Z' => {
                current.push(c);
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        current.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(match current.as_str() {
                    "const" => Token::Const,
                    "void" => Token::Void,
                    "int" => Token::Int,
                    "char" => Token::Char,
                    "double" => Token::Double,
                    "struct" => Token::Struct,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "switch" => Token::Switch,
                    "case" => Token::Case,
                    "default" => Token::Default,
                    "while" => Token::While,
                    "for" => Token::For,
                    "do" => Token::Do,
                    "return" => Token::Return,
                    "break" => Token::Break,
                    "continue" => Token::Continue,
                    "print" => Token::Print,
                    "scan" => Token::Scan,
                    _ => Token::Identifier(current.clone()),
                });
                current.clear();
            }
            '0'..='9' => {
                current.push(c);
                while let Some(&next) = chars.peek() {
                    if next.is_digit(10) || next == '.' {
                        current.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(current.parse::<f64>().unwrap()));
                current.clear();
            }
            '"' => in_string = true,
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            '{' => tokens.push(Token::LeftBrace),
            '}' => tokens.push(Token::RightBrace),
            '.' => tokens.push(Token::Dot),
            ',' => tokens.push(Token::Comma),
            ';' => tokens.push(Token::Semicolon),
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            '<' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::LessEqual);
                } else {
                    tokens.push(Token::Less);
                }
            }
            '>' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::GreaterEqual);
                } else {
                    tokens.push(Token::Greater);
                }
            }
            '=' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::EqualEqual);
                } else {
                    tokens.push(Token::Equal);
                }
            }
            '!' => {
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::BangEqual);
                } else {
                    tokens.push(Token::Bang);
                }
            }
            ' ' | '\t' | '\r' | '\n' => {} // Ignore whitespace
            _ => {
                eprintln!("Unexpected character: {}", c);
            }
        }
    }
    tokens.push(Token::Eof);
    tokens
}
