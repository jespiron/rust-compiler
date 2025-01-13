use std::fs::File;
use std::io::{BufReader, Read};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq)]
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

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    current: String,
    in_string: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
            current: String::new(),
            in_string: false,
        }
    }

    pub fn from_file(file: File) -> Result<Lexer<'static>, std::io::Error> {
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;
        // Note: We need to leak the String to get a 'static lifetime.
        // In a production environment, you might want to handle this differently.
        let contents = Box::leak(contents.into_boxed_str());
        Ok(Lexer::new(contents))
    }

    fn read_identifier(&mut self, first_char: char) -> Token {
        self.current.clear();
        self.current.push(first_char);

        while let Some(&next) = self.chars.peek() {
            if next.is_alphanumeric() || next == '_' {
                self.current.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        match self.current.as_str() {
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
            _ => Token::Identifier(self.current.clone()),
        }
    }

    fn read_number(&mut self, first_char: char) -> Token {
        self.current.clear();
        self.current.push(first_char);

        while let Some(&next) = self.chars.peek() {
            if next.is_digit(10) || next == '.' {
                self.current.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        Token::Number(self.current.parse::<f64>().unwrap())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = self.chars.next() {
            if self.in_string {
                if c == '"' {
                    let token = Token::StringLiteral(std::mem::take(&mut self.current));
                    self.in_string = false;
                    return Some(token);
                } else {
                    self.current.push(c);
                    continue;
                }
            }

            return Some(match c {
                'a'..='z' | 'A'..='Z' => self.read_identifier(c),
                '0'..='9' => self.read_number(c),
                '"' => {
                    self.in_string = true;
                    self.current.clear();
                    continue;
                }
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                '.' => Token::Dot,
                ',' => Token::Comma,
                ';' => Token::Semicolon,
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '/' => Token::Slash,
                '<' => {
                    if let Some('=') = self.chars.peek() {
                        self.chars.next();
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                }
                '>' => {
                    if let Some('=') = self.chars.peek() {
                        self.chars.next();
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                }
                '=' => {
                    if let Some('=') = self.chars.peek() {
                        self.chars.next();
                        Token::EqualEqual
                    } else {
                        Token::Equal
                    }
                }
                '!' => {
                    if let Some('=') = self.chars.peek() {
                        self.chars.next();
                        Token::BangEqual
                    } else {
                        Token::Bang
                    }
                }
                ' ' | '\t' | '\r' | '\n' => continue,
                _ => {
                    eprintln!("Unexpected character: {}", c);
                    continue;
                }
            });
        }

        Some(Token::Eof)
    }
}
