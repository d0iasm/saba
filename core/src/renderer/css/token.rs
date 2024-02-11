//! This is a part of "3. Tokenizing and Parsing CSS" in the "CSS Syntax Module Level 3" spec.
//! https://www.w3.org/TR/css-syntax-3/#tokenizing-and-parsing
//!
//! 4. Tokenization
//! https://www.w3.org/TR/css-syntax-3/#tokenization

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
/// https://www.w3.org/TR/css-syntax-3/#consume-token
/// https://www.w3.org/TR/css-syntax-3/#tokenization
pub enum CssToken {
    /// https://www.w3.org/TR/css-syntax-3/#typedef-hash-token
    HashToken(String),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-delim-token
    Delim(char),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-number-token
    Number(f64),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-colon-token
    Colon,
    /// https://www.w3.org/TR/css-syntax-3/#typedef-semicolon-token
    SemiColon,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-open-paren
    OpenParenthesis,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-close-paren
    CloseParenthesis,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-open-curly
    OpenCurly,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-close-curly
    CloseCurly,
    /// https://www.w3.org/TR/css-syntax-3/#typedef-ident-token
    Ident(String),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-string-token
    StringToken(String),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-at-keyword-token
    AtKeyword(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssTokenizer {
    pos: usize,
    input: Vec<char>,
}

impl CssTokenizer {
    pub fn new(css: String) -> Self {
        Self {
            pos: 0,
            input: css.chars().collect(),
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-name
    fn consume_ident_token(&mut self) -> String {
        let mut s = String::new();
        s.push(self.input[self.pos]);

        loop {
            self.pos += 1;
            let c = self.input[self.pos];
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => {
                    s.push(c);
                }
                _ => break,
            }
        }

        s
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-a-string-token
    fn consume_string_token(&mut self) -> String {
        let mut s = String::new();

        loop {
            if self.pos >= self.input.len() {
                return s;
            }

            self.pos += 1;
            let c = self.input[self.pos];
            match c {
                '"' => break,
                _ => s.push(c),
            }
        }

        s
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-number
    /// https://www.w3.org/TR/css-syntax-3/#consume-a-numeric-token
    fn consume_numeric_token(&mut self) -> f64 {
        let mut num = 0f64;
        let mut floating = false;
        let mut floating_digit = 1f64;

        loop {
            if self.pos >= self.input.len() {
                return num;
            }

            let c = self.input[self.pos];

            match c {
                '0'..='9' => {
                    if floating {
                        floating_digit *= 1f64 / 10f64;
                        num += (c.to_digit(10).unwrap() as f64) * floating_digit
                    } else {
                        num = num * 10.0 + (c.to_digit(10).unwrap() as f64);
                    }
                    self.pos += 1;
                }
                '.' => {
                    floating = true;
                    self.pos += 1;
                }
                _ => break,
            }
        }

        num
    }
}

impl Iterator for CssTokenizer {
    type Item = CssToken;

    /// https://www.w3.org/TR/css-syntax-3/#consume-token
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos >= self.input.len() {
                return None;
            }

            let c = self.input[self.pos];

            let token = match c {
                '"' | '\'' => {
                    let value = self.consume_string_token();
                    CssToken::StringToken(value)
                }
                '#' => {
                    // TODO: support the case if the next token is not ident code point
                    // "Otherwise, return a <delim-token> with its value set to the current input
                    // code point."
                    // https://www.w3.org/TR/css-syntax-3/#consume-token
                    let value = self.consume_ident_token();
                    self.pos -= 1;
                    CssToken::HashToken(value)
                }
                '(' => CssToken::OpenParenthesis,
                ')' => CssToken::CloseParenthesis,
                ',' => CssToken::Delim(','),
                // TODO: support minus number with hyphen.
                // "If the input stream starts with a number, reconsume the current input code
                // point, consume a numeric token, and return it."
                // https://www.w3.org/TR/css-syntax-3/#consume-a-token
                '-' => {
                    let t = CssToken::Ident(self.consume_ident_token());
                    self.pos -= 1;
                    t
                }
                // TODO: support floating number case.
                // "If the input stream starts with a number, reconsume the current input code
                // point, consume a numeric token, and return it."
                // https://www.w3.org/TR/css-syntax-3/#consume-a-token
                '.' => CssToken::Delim('.'),
                ':' => CssToken::Colon,
                ';' => CssToken::SemiColon,
                '@' => {
                    // If the next 3 input code points would start an ident sequence, consume an
                    // ident sequence, create an <at-keyword-token> with its value set to the
                    // returned value, and return it.
                    if self.input[self.pos + 1].is_ascii_alphabetic()
                        && self.input[self.pos + 2].is_alphanumeric()
                    {
                        // skip '@'
                        self.pos += 1;
                        let t = CssToken::AtKeyword(self.consume_ident_token());
                        self.pos -= 1;
                        t
                    } else {
                        CssToken::Delim('@')
                    }
                }
                '{' => CssToken::OpenCurly,
                '}' => CssToken::CloseCurly,
                // digit
                // Reconsume the current input code point, consume a numeric token, and return it.
                '0'..='9' => {
                    let t = CssToken::Number(self.consume_numeric_token());
                    self.pos -= 1;
                    t
                }
                // ident-start code point
                // Reconsume the current input code point, consume an ident-like token, and return
                // it.
                'a'..='z' | 'A'..='Z' | '_' => {
                    let t = CssToken::Ident(self.consume_ident_token());
                    self.pos -= 1;
                    t
                }
                // TODO: handle white spaces property
                // "Consume as much whitespace as possible. Return a <whitespace-token>."
                // https://www.w3.org/TR/css-syntax-3/#consume-token
                ' ' | '\n' => {
                    self.pos += 1;
                    continue;
                }
                _ => {
                    /*
                    console_error(
                        self.browser.clone(),
                        format!("char {} is not supported yet", c),
                    );
                    self.pos += 1;
                    continue;
                    */
                    panic!("char {} is not supported yet", c);
                }
            };

            self.pos += 1;
            return Some(token);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_empty() {
        let style = "".to_string();
        let mut t = CssTokenizer::new(style);
        assert!(t.next().is_none());
    }

    #[test]
    fn test_one_rule() {
        let style = "p { color: red; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Ident("p".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }

    #[test]
    fn test_multiple_rules() {
        // The value like "40px" is not supported yet.
        let style = "p { color: red; } h1 { font-size: 40; color: blue; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Ident("p".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
            CssToken::Ident("h1".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("font-size".to_string()),
            CssToken::Colon,
            CssToken::Number(40.0),
            CssToken::SemiColon,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("blue".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }
}
