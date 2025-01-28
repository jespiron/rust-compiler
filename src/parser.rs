use crate::lexer::Token;
use std::fmt;

// Program is comprised of variables and functions
#[derive(Debug)]
pub struct Program {
    pub decl: Vec<VarDeclaration>,
    pub fns: Vec<FnDeclaration>,
}

// Example: `const int my_variable = !(2+3)`
#[derive(Debug)]
pub struct VarDeclaration {
    pub is_const: bool,    // true
    pub type_token: Token, // `int`
    pub identifier: Token, // `my_variable`
    pub value: Expr,       // Unary(Bang, Parentheses(Binary(Number(2.0), Plus, Number(2.0))))
}

// Function declaration with parameters and body
#[derive(Debug)]
pub struct FnDeclaration {
    pub return_type: Token,
    pub identifier: Token,
    pub params: Vec<Parameter>,
    pub body: Block,
}

// Function parameter
#[derive(Debug)]
pub struct Parameter {
    pub type_token: Token,
    pub identifier: Token,
}

// Block of statements
#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
}

// Different types of statements
#[derive(Debug)]
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

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken { found: Token, expected: Vec<Token> },
    UnexpectedEOF { expected: Vec<Token> },
    InvalidExpression,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::UnexpectedToken { found, expected } => {
                write!(
                    f,
                    "Unexpected token: {:?}. Expected one of: {:?}",
                    found, expected
                )
            }
            ParserError::UnexpectedEOF { expected } => {
                write!(f, "Unexpected EOF. Expected one of: {:?}", expected)
            }
            ParserError::InvalidExpression {} => {
                write!(f, "Invalid expression")
            }
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut declarations = Vec::new();
        let mut functions = Vec::new();

        while !self.is_at_end() {
            if self.match_token(&[Token::Const]) {
                declarations.push(self.variable_declaration(true)?);
            } else if self.check_type_token() {
                if self.peek_ahead_for_lparen() {
                    functions.push(self.function_declaration()?);
                } else {
                    declarations.push(self.variable_declaration(false)?);
                }
            } else {
                // Error handling could be added here
                self.advance();
            }
        }

        Ok(Program {
            decl: declarations,
            fns: functions,
        })
    }

    fn variable_declaration(&mut self, is_const: bool) -> Result<VarDeclaration, ParserError> {
        let type_token = self.advance(); // Type token
        let identifier = self.consume_identifier()?;

        self.consume(&Token::Equal)?; // Expect '='
        let value = self.expression()?;
        self.consume(&Token::Semicolon)?;

        Ok(VarDeclaration {
            is_const,
            type_token,
            identifier,
            value,
        })
    }

    fn function_declaration(&mut self) -> Result<FnDeclaration, ParserError> {
        let return_type = self.advance(); // Type token
        let identifier = self.consume_identifier()?;

        self.consume(&Token::LeftParen);
        let params = self.parameters()?;
        self.consume(&Token::RightParen);

        let body = self.block()?;

        Ok(FnDeclaration {
            return_type,
            identifier,
            params,
            body,
        })
    }

    fn parameters(&mut self) -> Result<Vec<Parameter>, ParserError> {
        let mut params = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                let type_token = self.consume_type()?;
                let identifier = self.consume_identifier()?;

                params.push(Parameter {
                    type_token,
                    identifier,
                });

                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }

        Ok(params)
    }

    fn block(&mut self) -> Result<Block, ParserError> {
        self.consume(&Token::LeftBrace);
        let mut statements = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }

        self.consume(&Token::RightBrace);
        Ok(Block { statements })
    }

    fn statement(&mut self) -> Result<Statement, ParserError> {
        if self.match_token(&[Token::If]) {
            self.if_statement()
        } else if self.match_token(&[Token::While]) {
            self.while_statement()
        } else if self.match_token(&[Token::Return]) {
            self.return_statement()
        } else if self.match_token(&[Token::Break]) {
            self.consume(&Token::Semicolon);
            Ok(Statement::Break)
        } else if self.match_token(&[Token::Continue]) {
            self.consume(&Token::Semicolon);
            Ok(Statement::Continue)
        } else if self.match_token(&[Token::Print]) {
            self.print_statement()
        } else if self.check(&Token::LeftBrace) {
            Ok(Statement::Block(self.block()?))
        } else if self.check_type_token() {
            Ok(Statement::VarDecl(self.variable_declaration(false)?))
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume(&Token::LeftParen);
        let condition = self.expression()?;
        self.consume(&Token::RightParen);

        let then_branch = Box::new(self.statement()?);

        let else_branch = if self.peek() == Token::Else {
            self.advance(); // consume the 'else' token
            Some(Box::new(self.statement()?))
        } else if !matches!(self.peek(), Token::RightBrace) {
            // if we're not at the end of a block, treat the next statement as an implicit else
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Statement::If(Box::new(condition), then_branch, else_branch))
    }

    fn while_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume(&Token::LeftParen)?;
        let condition = self.expression()?;
        self.consume(&Token::RightParen)?;
        let body = self.statement()?;
        Ok(Statement::While(Box::new(condition), Box::new(body)))
    }

    fn return_statement(&mut self) -> Result<Statement, ParserError> {
        let value = if !self.check(&Token::Semicolon) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        self.consume(&Token::Semicolon);
        Ok(Statement::Return(value))
    }

    fn print_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume(&Token::LeftParen);
        let expr = self.expression()?;
        self.consume(&Token::RightParen);
        self.consume(&Token::Semicolon);
        Ok(Statement::Print(Box::new(expr)))
    }

    fn expression_statement(&mut self) -> Result<Statement, ParserError> {
        let expr = self.expression()?;
        self.consume(&Token::Semicolon);
        Ok(Statement::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.equality()?;

        if self.match_token(&[Token::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Binary(
                    Box::new(Expr::Variable(name)),
                    equals,
                    Box::new(value),
                ));
            }
            return Err(ParserError::InvalidExpression);
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[Token::BangEqual, Token::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term()?;

        while self.match_token(&[
            Token::Greater,
            Token::GreaterEqual,
            Token::Less,
            Token::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;

        while self.match_token(&[Token::Plus, Token::Minus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;

        while self.match_token(&[Token::Star, Token::Slash]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.match_token(&[Token::Bang, Token::Minus, Token::Tilde]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        let token = self.peek();
        match token {
            Token::Number(_) | Token::StringLiteral(_) => {
                self.advance();
                Ok(Expr::Literal(token))
            }
            Token::Identifier(_) => {
                let identifier = self.advance();
                if self.match_token(&[Token::LeftParen]) {
                    let args = self.arguments()?;
                    self.consume(&Token::RightParen);
                    Ok(Expr::Call(Box::new(Expr::Variable(identifier)), args))
                } else {
                    Ok(Expr::Variable(identifier))
                }
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(&Token::RightParen);
                Ok(Expr::Parentheses(Box::new(expr)))
            }
            _ => Err(ParserError::UnexpectedToken {
                found: token.clone(),
                expected: vec![
                    // Alternative is to define a ExpectedToken enum,
                    // which is the same as Token except no parameters.
                    // This way, we won't need to pass placeholder parameters here.
                    // However, this means that we have to sync ExpectedToken with Token,
                    // which sounds like too much work for the sake of pretty error messages.
                    Token::Number(0.0),
                    Token::StringLiteral(String::from("placeholder")),
                    Token::Identifier(String::from("placeholder")),
                    Token::LeftParen,
                ],
            }),
        }
    }

    fn arguments(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut args = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                args.push(self.expression()?);
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }

        Ok(args)
    }

    // Helper methods
    fn match_token(&mut self, tokens: &[Token]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }

        match (token, &self.peek()) {
            // Match variants regardless of their contained values
            (Token::Number(_), Token::Number(_))
            | (Token::StringLiteral(_), Token::StringLiteral(_))
            | (Token::Identifier(_), Token::Identifier(_)) => true,
            // For all other tokens, exact match
            (t1, t2) => std::mem::discriminant(t1) == std::mem::discriminant(t2),
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek() == Token::Eof
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token: &Token) -> Result<(), ParserError> {
        if self.check(token) {
            self.advance();
            return Ok(());
        }
        if self.is_at_end() {
            return Err(ParserError::UnexpectedEOF {
                expected: vec![token.clone()],
            });
        }
        Err(ParserError::UnexpectedToken {
            found: self.peek(),
            expected: vec![token.clone()],
        })
    }

    fn consume_identifier(&mut self) -> Result<Token, ParserError> {
        match &self.peek() {
            Token::Identifier(_) => Ok(self.advance()),
            _ => Err(ParserError::UnexpectedToken {
                found: self.peek(),
                expected: vec![Token::Identifier(String::from("placeholder"))],
            }),
        }
    }

    fn consume_type(&mut self) -> Result<Token, ParserError> {
        if self.check_type_token() {
            Ok(self.advance())
        } else {
            Err(ParserError::UnexpectedToken {
                found: self.peek(),
                expected: vec![
                    Token::Int,
                    Token::Char,
                    Token::Double,
                    Token::Void,
                    Token::Struct,
                ],
            })
        }
    }

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
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, ParserError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
