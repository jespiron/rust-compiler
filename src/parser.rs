// Lexer: charstream -> tokens
// Parser: tokens -> ast

#[derive(Debug)]
pub struct AstNode {}

pub struct Lexer {}

impl Lexer {
    pub fn tokenize(self, charstream: &str) -> Vec<String> {
        vec![]
    }
}

pub struct Parser {}

impl Parser {
    pub fn parse(self, tokens: Vec<String>) -> AstNode {
        AstNode {}
    }
}
