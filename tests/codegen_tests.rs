use rust_compiler::codegen::generate_code;
use rust_compiler::lexer::Token;
use rust_compiler::parser::{
    Block, Expr, FnDeclaration, Parameter, Program, Statement, VarDeclaration,
};

#[test]
fn test_sample_program() {
    // Create AST for:
    // int g0 = 42;
    // double g1 = 1.0;
    //
    // int fun(int num) {
    //     return -num;
    // }
    //
    // int main() {
    //     return fun(-123456);
    // }

    let program = Program {
        decl: vec![
            // int g0 = 42
            VarDeclaration {
                is_const: false,
                type_token: Token::Int,
                identifier: Token::Identifier(String::from("g0")),
                value: Expr::Literal(Token::Number(42.0)),
            },
            // double g1 = 1.0
            VarDeclaration {
                is_const: false,
                type_token: Token::Number(1.0),
                identifier: Token::Identifier(String::from("g1")),
                value: Expr::Literal(Token::Number(1.0)),
            },
        ],
        fns: vec![
            // int fun(int num)
            FnDeclaration {
                return_type: Token::Int,
                identifier: Token::Identifier(String::from("fun")),
                params: vec![Parameter {
                    type_token: Token::Int,
                    identifier: Token::Identifier(String::from("num")),
                }],
                body: Block {
                    statements: vec![
                        // return -num;
                        Statement::Return(Some(Box::new(Expr::Unary(
                            Token::Minus,
                            Box::new(Expr::Variable(Token::Identifier(String::from("num")))),
                        )))),
                    ],
                },
            },
            // int main()
            FnDeclaration {
                return_type: Token::Int,
                identifier: Token::Identifier(String::from("main")),
                params: vec![],
                body: Block {
                    statements: vec![
                        // return fun(-123456);
                        Statement::Return(Some(Box::new(Expr::Call(
                            Box::new(Expr::Variable(Token::Identifier(String::from("fun")))),
                            vec![Expr::Literal(Token::Number(-123456.0))],
                        )))),
                    ],
                },
            },
        ],
    };
}
