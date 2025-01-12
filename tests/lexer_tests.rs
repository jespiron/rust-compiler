use rust_compiler::lexer::{tokenize_from_string, Token};

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

        let tokens = tokenize_from_string(source);

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

        assert_eq!(tokens, expected_tokens);
    }
}
