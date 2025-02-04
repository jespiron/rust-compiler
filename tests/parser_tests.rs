#[cfg(test)]
mod tests {
    use rust_compiler::lexer::Token;
    use rust_compiler::parser::{parse, Expr, Program, Statement};

    #[test]
    fn test_hello_world() {
        let tokens = vec![
            Token::Int,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::Identifier("printf".to_string()),
            Token::LeftParen,
            Token::StringLiteral("Hello, world!\n".to_string()),
            Token::RightParen,
            Token::Semicolon,
            Token::Return,
            Token::Number(0.0),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        let program = parse(tokens).unwrap();

        assert_eq!(program.decl.len(), 0);
        assert_eq!(program.fns.len(), 1);

        let main_fn = &program.fns[0];
        assert_eq!(main_fn.params.len(), 0);
        assert_eq!(main_fn.identifier, Token::Identifier("main".to_string()));
        assert_eq!(main_fn.return_type, Token::Int);

        let statements = &main_fn.body.statements;
        assert_eq!(statements.len(), 2); // printf and return
    }

    #[test]
    fn test_variable_declaration() {
        let tokens = vec![
            Token::Const,
            Token::Int,
            Token::Identifier("MAX_SIZE".to_string()),
            Token::Equal,
            Token::Number(100.0),
            Token::Semicolon,
            Token::Eof,
        ];

        let program = parse(tokens).unwrap();

        assert_eq!(program.decl.len(), 1);
        assert_eq!(program.fns.len(), 0);

        let var_decl = &program.decl[0];
        assert!(var_decl.is_const);
        assert_eq!(var_decl.type_token, Token::Int);
        assert_eq!(
            var_decl.identifier,
            Token::Identifier("MAX_SIZE".to_string())
        );

        match &var_decl.value {
            Expr::Literal(Token::Number(n)) => assert_eq!(*n, 100.0),
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_function_with_parameters() {
        let tokens = vec![
            Token::Int,
            Token::Identifier("add".to_string()),
            Token::LeftParen,
            Token::Int,
            Token::Identifier("a".to_string()),
            Token::Comma,
            Token::Int,
            Token::Identifier("b".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::Identifier("a".to_string()),
            Token::Plus,
            Token::Identifier("b".to_string()),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        let program = parse(tokens).unwrap();

        assert_eq!(program.fns.len(), 1);

        let add_fn = &program.fns[0];
        assert_eq!(add_fn.params.len(), 2);
        assert_eq!(add_fn.identifier, Token::Identifier("add".to_string()));

        let param1 = &add_fn.params[0];
        assert_eq!(param1.type_token, Token::Int);
        assert_eq!(param1.identifier, Token::Identifier("a".to_string()));

        let param2 = &add_fn.params[1];
        assert_eq!(param2.type_token, Token::Int);
        assert_eq!(param2.identifier, Token::Identifier("b".to_string()));
    }

    #[test]
    fn test_if_statement() {
        let tokens = vec![
            Token::Int,
            Token::Identifier("abs".to_string()),
            Token::LeftParen,
            Token::Int,
            Token::Identifier("x".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::If,
            Token::LeftParen,
            Token::Identifier("x".to_string()),
            Token::Less,
            Token::Number(0.0),
            Token::RightParen,
            Token::Return,
            Token::Minus,
            Token::Identifier("x".to_string()),
            Token::Semicolon,
            Token::Return,
            Token::Identifier("x".to_string()),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        let program = parse(tokens).unwrap();

        let abs_fn = &program.fns[0];
        assert_eq!(abs_fn.identifier, Token::Identifier("abs".to_string()));

        let statements = &abs_fn.body.statements;
        match &statements[0] {
            Statement::If(condition, then_branch, else_branch) => {
                match &**condition {
                    Expr::Binary(left, op, right) => {
                        match &**left {
                            Expr::Variable(Token::Identifier(name)) => assert_eq!(name, "x"),
                            _ => panic!("Expected variable reference"),
                        }
                        assert_eq!(*op, Token::Less);
                        match &**right {
                            Expr::Literal(Token::Number(n)) => assert_eq!(*n, 0.0),
                            _ => panic!("Expected number literal"),
                        }
                    }
                    _ => panic!("Expected binary expression"),
                }
                assert!(else_branch.is_none());
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_while_loop() {
        let tokens = vec![
            Token::Void,
            Token::Identifier("countdown".to_string()),
            Token::LeftParen,
            Token::Int,
            Token::Identifier("n".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::While,
            Token::LeftParen,
            Token::Identifier("n".to_string()),
            Token::Greater,
            Token::Number(0.0),
            Token::RightParen,
            Token::LeftBrace,
            Token::Identifier("n".to_string()),
            Token::Equal,
            Token::Identifier("n".to_string()),
            Token::Minus,
            Token::Number(1.0),
            Token::Semicolon,
            Token::RightBrace,
            Token::RightBrace,
            Token::Eof,
        ];

        let program = parse(tokens).unwrap();

        let countdown_fn = &program.fns[0];
        assert_eq!(countdown_fn.return_type, Token::Void);

        let statements = &countdown_fn.body.statements;
        match &statements[0] {
            Statement::While(condition, body) => match &**condition {
                Expr::Binary(left, op, right) => {
                    match &**left {
                        Expr::Variable(Token::Identifier(name)) => assert_eq!(name, "n"),
                        _ => panic!("Expected variable reference"),
                    }
                    assert_eq!(*op, Token::Greater);
                    match &**right {
                        Expr::Literal(Token::Number(n)) => assert_eq!(*n, 0.0),
                        _ => panic!("Expected number literal"),
                    }
                }
                _ => panic!("Expected binary expression"),
            },
            _ => panic!("Expected while statement"),
        }
    }
}
