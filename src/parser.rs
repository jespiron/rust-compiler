use crate::lexer::Token;

// Program is comprised of variables and functions
pub struct Program {
    pub decl: Vec<VarDeclaration>,
    pub fns: Vec<FnDeclaration>,
}

// Example: `const int my_variable = !(2+3)`
pub struct VarDeclaration {
    pub is_const: bool,    // true
    pub type_token: Token, // `int`
    pub identifier: Token, // `my_variable`
    pub value: Expr,       // Unary(Bang, Parentheses(Binary(Number(2.0), Plus, Number(2.0))))
}

// Function declaration with parameters and body
pub struct FnDeclaration {
    pub return_type: Token,
    pub identifier: Token,
    pub params: Vec<Parameter>,
    pub body: Block,
}

// Function parameter
pub struct Parameter {
    pub type_token: Token,
    pub identifier: Token,
}

// Block of statements
pub struct Block {
    pub statements: Vec<Statement>,
}

// Different types of statements
pub enum Statement {
    Expression(Expr),
    VarDecl(VarDeclaration),
    If(Box<Expr>, Box<Statement>, Option<Box<Statement>>), // condition, then-branch, else-branch
    While(Box<Expr>, Box<Statement>),
    Return(Option<Box<Expr>>),
    Block(Block),
    Print(Box<Expr>),
    Break,
    Continue,
}

#[derive(Debug)]
pub enum Expr {
    Literal(Token),                      // leaf node of the expression tree
    Unary(Token, Box<Expr>),             // like `!expression`
    Binary(Box<Expr>, Token, Box<Expr>), // like `2+3`
    Parentheses(Box<Expr>),              // like `(expression)`
    Variable(Token),                     // variable reference
    Call(Box<Expr>, Vec<Expr>),          // function call with arguments
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Program {
        let mut declarations = Vec::new();
        let mut functions = Vec::new();

        while !self.is_at_end() {
            if self.match_token(&[Token::Const]) {
                declarations.push(self.variable_declaration(true));
            } else if self.check_type_token() {
                // Look ahead to see if it's a function or variable declaration
                if self.peek_ahead_for_lparen() {
                    functions.push(self.function_declaration());
                } else {
                    declarations.push(self.variable_declaration(false));
                }
            }
        }

        Program {
            decl: declarations,
            fns: functions,
        }
    }

    fn variable_declaration(&mut self, is_const: bool) -> VarDeclaration {
        let type_token = self.advance(); // Type token
        let identifier = self.consume_identifier();

        self.consume(&Token::Equal); // Expect '='
        let value = self.expression();
        self.consume(&Token::Semicolon);

        VarDeclaration {
            is_const,
            type_token,
            identifier,
            value,
        }
    }

    fn function_declaration(&mut self) -> FnDeclaration {
        let return_type = self.advance();
        let identifier = self.consume_identifier();

        self.consume(&Token::LeftParen);
        let params = self.parameters();
        self.consume(&Token::RightParen);

        let body = self.block();

        FnDeclaration {
            return_type,
            identifier,
            params,
            body,
        }
    }

    // Helper methods
    fn check_type_token(&self) -> bool {
        matches!(
            self.peek(),
            Token::Int | Token::Char | Token::Double | Token::Void | Token::Struct
        )
    }

    fn peek_ahead_for_lparen(&self) -> bool {
        let mut i = self.current;
        while i < self.tokens.len() {
            match self.tokens[i] {
                Token::LeftParen => return true,
                Token::Semicolon => return false,
                _ => i += 1,
            }
        }
        false
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_token(&[Token::BangEqual, Token::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        expr
    }
}

pub fn parse(tokens: Vec<Token>) -> Program {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
