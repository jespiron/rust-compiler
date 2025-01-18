// Parser: tokens -> ast
use crate::lexer::Token;

// Program is comprised of variables and functions
pub struct Program {
    pub decl: Vec<VarDeclaration>,
    pub fns: Vec<FnDeclaration>,
}

// Example: `const int my_variable = !(2+3)`
pub struct VarDeclaration {
    is_const: bool,    // true
    _type: Token,      // `int`
    identifier: Token, // `my_variable`
    value: Expr,       // Unary(Bang, Parentheses(Binary(Number(2.0), Plus, Number(2.0))))
}

// TODO
pub struct FnDeclaration {
    return_type: Token,
}

#[derive(Debug)]
pub enum Expr {
    Literal(Token),                      // leaf node of the expression tree
    Unary(Token, Box<Expr>),             // like `!expression`
    Binary(Box<Expr>, Token, Box<Expr>), // like `2+3`
    Parentheses(Box<Expr>),              // like `(expression)`
}

pub fn parse(tokens: Vec<Token>) -> Program {
    let decl = vec![];
    let fns = vec![];
    Program { decl, fns }
}
