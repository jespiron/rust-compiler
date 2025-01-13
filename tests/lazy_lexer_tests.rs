use rust_compiler::lazy_lexer::{Lexer, Token};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_simple() {
        let source = r#"
        int main() {
            printf("Hello, world!\n");
            return 0;
        }
        "#;

        let mut tokens = Lexer::new(source);

        let expected_tokens = vec![
            Token::Int,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::Identifier("printf".to_string()),
            Token::LeftParen,
            Token::StringLiteral("Hello, world!\\n".to_string()),
            Token::RightParen,
            Token::Semicolon,
            Token::Return,
            Token::Number(0.0),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        for expected_token in expected_tokens {
            let token = tokens
                .next()
                .expect("Lexer terminated early. Expected more tokens");
            assert_eq!(token, expected_token);
        }
    }
}
