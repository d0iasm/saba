//! https://262.ecma-international.org/12.0/#sec-ecmascript-language-lexical-grammar

use alloc::string::{String, ToString};
use alloc::vec::Vec;

static RESERVED_WORDS: [&str; 3] = ["var", "function", "return"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// https://262.ecma-international.org/12.0/#sec-identifier-names
    Identifier(String),
    /// https://262.ecma-international.org/12.0/#sec-keywords-and-reserved-words
    Keyword(String),
    /// https://262.ecma-international.org/12.0/#sec-punctuators
    Punctuator(char),
    /// https://262.ecma-international.org/12.0/#sec-literals-string-literals
    StringLiteral(String),
    /// https://262.ecma-international.org/12.0/#sec-literals-numeric-literals
    Number(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsLexer {
    pos: usize,
    input: Vec<char>,
}

impl JsLexer {
    pub fn new(js: String) -> Self {
        Self {
            pos: 0,
            input: js.chars().collect(),
        }
    }

    fn consume_number(&mut self) -> u64 {
        let mut num = 0;

        loop {
            if self.pos >= self.input.len() {
                return num;
            }

            let c = self.input[self.pos];

            match c {
                '0'..='9' => {
                    num = num * 10 + (c.to_digit(10).unwrap() as u64);
                    self.pos += 1;
                }
                _ => break,
            }
        }

        return num;
    }

    fn consume_string(&mut self) -> String {
        let mut result = String::new();
        self.pos += 1;

        loop {
            if self.pos >= self.input.len() {
                return result;
            }

            if self.input[self.pos] == '"' {
                self.pos += 1;
                return result;
            }

            result.push(self.input[self.pos]);
            self.pos += 1;
        }
    }

    fn consume_identifier(&mut self) -> String {
        let mut result = String::new();

        loop {
            if self.pos >= self.input.len() {
                return result;
            }

            // https://262.ecma-international.org/12.0/#prod-IdentifierPart
            if self.input[self.pos].is_ascii_alphanumeric() || self.input[self.pos] == '$' {
                result.push(self.input[self.pos]);
                self.pos += 1;
            } else {
                return result;
            }
        }
    }

    fn contains(&self, keyword: &str) -> bool {
        for i in 0..keyword.len() {
            if keyword
                .chars()
                .nth(i)
                .expect("failed to access to i-th char")
                != self.input[self.pos + i]
            {
                return false;
            }
        }

        true
    }

    fn check_reserved_word(&self) -> Option<String> {
        for word in RESERVED_WORDS {
            if self.contains(word) {
                return Some(word.to_string());
            }
        }

        None
    }

    pub fn peek(&mut self) -> Option<Token> {
        let start_position = self.pos;

        let token = self.get_next_token();

        // Restore the start position to avoid consuming input.
        self.pos = start_position;

        token
    }

    fn get_next_token(&mut self) -> Option<Token> {
        if self.pos >= self.input.len() {
            return None;
        }

        // skip a white space and a new line
        while self.input[self.pos] == ' ' || self.input[self.pos] == '\n' {
            self.pos += 1;

            if self.pos >= self.input.len() {
                return None;
            }
        }

        match self.check_reserved_word() {
            Some(keyword) => {
                self.pos += keyword.len();
                let token = Some(Token::Keyword(keyword));
                return token;
            }
            None => {}
        }

        let c = self.input[self.pos];

        let token = match c {
            '+' | '-' | ';' | '=' | '(' | ')' | '{' | '}' | ',' | '.' => {
                let t = Token::Punctuator(c);
                self.pos += 1;
                t
            }
            '"' => Token::StringLiteral(self.consume_string()),
            '0'..='9' => Token::Number(self.consume_number()),
            // https://262.ecma-international.org/12.0/#prod-IdentifierStart
            'a'..='z' | 'A'..='Z' | '_' | '$' => Token::Identifier(self.consume_identifier()),
            _ => unimplemented!("char {:?} is not supported yet", c),
        };

        Some(token)
    }
}

impl Iterator for JsLexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let input = "".to_string();
        let mut lexer = JsLexer::new(input);
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_num() {
        let input = "42".to_string();
        let mut lexer = JsLexer::new(input);
        let expected = [Token::Number(42)].to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_string() {
        let input = "\"foo\"".to_string();
        let mut lexer = JsLexer::new(input);
        let expected = [Token::StringLiteral("foo".to_string())].to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_add_nums() {
        let input = "1 + 2".to_string();
        let mut lexer = JsLexer::new(input);
        let expected = [Token::Number(1), Token::Punctuator('+'), Token::Number(2)].to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_add_strings() {
        let input = "\"foo\" + \"bar\"".to_string();
        let mut lexer = JsLexer::new(input);
        let expected = [
            Token::StringLiteral("foo".to_string()),
            Token::Punctuator('+'),
            Token::StringLiteral("bar".to_string()),
        ]
        .to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_add_num_string() {
        let input = "1 + \"2\"".to_string();
        let mut lexer = JsLexer::new(input);
        let expected = [
            Token::Number(1),
            Token::Punctuator('+'),
            Token::StringLiteral("2".to_string()),
        ]
        .to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_assign_variable() {
        let input = "var foo=42;".to_string();
        let mut lexer = JsLexer::new(input);
        let expected = [
            Token::Keyword("var".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('='),
            Token::Number(42),
            Token::Punctuator(';'),
        ]
        .to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_add_variable_num() {
        let input = "var foo=42; var result=foo+1;".to_string();
        let mut lexer = JsLexer::new(input);
        let expected = [
            Token::Keyword("var".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('='),
            Token::Number(42),
            Token::Punctuator(';'),
            Token::Keyword("var".to_string()),
            Token::Identifier("result".to_string()),
            Token::Punctuator('='),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('+'),
            Token::Number(1),
            Token::Punctuator(';'),
        ]
        .to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_define_function() {
        let input = "function foo() { return 42; }".to_string();
        let mut lexer = JsLexer::new(input);
        let expected = [
            Token::Keyword("function".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('('),
            Token::Punctuator(')'),
            Token::Punctuator('{'),
            Token::Keyword("return".to_string()),
            Token::Number(42),
            Token::Punctuator(';'),
            Token::Punctuator('}'),
        ]
        .to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_define_function_with_args() {
        let input = "function foo(a, b) { return 42; }".to_string();
        let mut lexer = JsLexer::new(input);
        let expected = [
            Token::Keyword("function".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('('),
            Token::Identifier("a".to_string()),
            Token::Punctuator(','),
            Token::Identifier("b".to_string()),
            Token::Punctuator(')'),
            Token::Punctuator('{'),
            Token::Keyword("return".to_string()),
            Token::Number(42),
            Token::Punctuator(';'),
            Token::Punctuator('}'),
        ]
        .to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_add_function_num() {
        let input = "function foo() { return 42; } var result = foo() + 1;".to_string();
        let mut lexer = JsLexer::new(input);
        let expected = [
            Token::Keyword("function".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('('),
            Token::Punctuator(')'),
            Token::Punctuator('{'),
            Token::Keyword("return".to_string()),
            Token::Number(42),
            Token::Punctuator(';'),
            Token::Punctuator('}'),
            Token::Keyword("var".to_string()),
            Token::Identifier("result".to_string()),
            Token::Punctuator('='),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('('),
            Token::Punctuator(')'),
            Token::Punctuator('+'),
            Token::Number(1),
            Token::Punctuator(';'),
        ]
        .to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }
}
