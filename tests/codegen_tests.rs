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

    let bytecode = generate_code(program);

    // Expected bytecode based on the provided example
    let expected = vec![
        // magic
        0x43, 0x30, 0x3A, 0x29, // version
        0x00, 0x00, 0x00, 0x01, // constants_count
        0x00, 0x06, // constants[0]: "fun"
        0x00, 0x00, 0x03, 0x66, 0x75, 0x6E, // constants[1]: "main"
        0x00, 0x00, 0x04, 0x6D, 0x61, 0x69, 0x6E, // constants[2]: -123456
        0x01, 0xFF, 0xFE, 0x1D, 0xC0, // constants[3]: 1.0
        0x02, 0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // start_code
        0x00, 0x02, 0x01, 0x2A, // bipush 42
        0x09, 0x00, 0x03, // loadc 3 (1.0)
        // functions_count
        0x00, 0x02, // function[0] (fun)
        0x00, 0x00, // name_index
        0x00, 0x01, // params_length
        0x00, 0x01, // level
        0x00, 0x04, // instructions_count
        0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // loada 0,0
        0x10, // iload
        0x40, // ineg
        0x89, // iret
        // function[1] (main)
        0x00, 0x01, // name_index
        0x00, 0x00, // params_length
        0x00, 0x01, // level
        0x00, 0x03, // instructions_count
        0x09, 0x00, 0x02, // loadc 2 (-123456)
        0x80, 0x00, 0x00, // call 0
        0x89, // iret
    ];

    assert_eq!(
        bytecode, expected,
        "\nExpected bytecode:\n{:02X?}\n\nGot bytecode:\n{:02X?}",
        expected, bytecode
    );
}
