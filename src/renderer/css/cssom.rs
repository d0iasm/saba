//! This is a part of "3. Tokenizing and Parsing CSS" in the "CSS Syntax Module Level 3" spec.
//! https://www.w3.org/TR/css-syntax-3/#tokenizing-and-parsing
//!
//! 5. Parsing
//! https://www.w3.org/TR/css-syntax-3/#parsing

use crate::renderer::css::token::*;

use std::string::String;
use std::string::ToString;
use std::vec::Vec;

// e.g.
// div {
//   background-color: green;
//   width: 100;
// }
// p {
//   color: red;
// }
//
// StyleSheet
// |-- QualifiedRule
//     |-- Selector
//         |-- div
//     |-- Vec<Declaration>
//         |-- background-color: green
//         |-- width: 100
// |-- QualifiedRule
//     |-- Selector
//         |-- p
//     |-- Vec<Declaration>
//         |-- color: red

#[derive(Debug, Clone, PartialEq)]
/// https://www.w3.org/TR/cssom-1/#cssstylesheet
pub struct StyleSheet {
    /// https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssrules
    pub rules: Vec<QualifiedRule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn set_rules(&mut self, rules: Vec<QualifiedRule>) {
        self.rules = rules;
    }
}

#[derive(Debug, Clone, PartialEq)]
// TODO: implement it properly
pub struct AtRule {
    // TODO: support list of media query
    /// https://www.w3.org/TR/mediaqueries-5/#typedef-media-query-list
    pub prelude: String,
    pub rule: QualifiedRule,
}

// TODO: support list of media query
impl AtRule {
    pub fn new() -> Self {
        Self {
            prelude: String::new(),
            rule: QualifiedRule::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// https://www.w3.org/TR/css-syntax-3/#qualified-rule
/// https://www.w3.org/TR/css-syntax-3/#style-rules
pub struct QualifiedRule {
    // TODO: support multiple selectors
    /// https://www.w3.org/TR/selectors-4/#typedef-selector-list
    /// The prelude of the qualified rule is parsed as a <selector-list>.
    pub selector: Selector,
    /// https://www.w3.org/TR/css-syntax-3/#parse-a-list-of-declarations
    /// The content of the qualified rule’s block is parsed as a list of declarations.
    pub declarations: Vec<Declaration>,
}

impl QualifiedRule {
    pub fn new() -> Self {
        Self {
            selector: Selector::TypeSelector("".to_string()),
            declarations: Vec::new(),
        }
    }

    pub fn set_selector(&mut self, selector: Selector) {
        self.selector = selector;
    }

    pub fn set_declarations(&mut self, declarations: Vec<Declaration>) {
        self.declarations = declarations;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// https://www.w3.org/TR/selectors-3/#selectors
pub enum Selector {
    /// https://www.w3.org/TR/selectors-3/#type-selectors
    TypeSelector(String),
    /// https://www.w3.org/TR/selectors-3/#class-html
    ClassSelector(String),
    /// https://www.w3.org/TR/selectors-3/#id-selectors
    IdSelector(String),
    /// This is an unofficial selector.
    UnknownSelector,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: ComponentValue,
}

/// https://www.w3.org/TR/css-syntax-3/#declaration
impl Declaration {
    pub fn new() -> Self {
        Self {
            property: String::new(),
            value: ComponentValue::Keyword(String::new()),
        }
    }

    pub fn set_property(&mut self, property: String) {
        self.property = property;
    }

    pub fn set_value(&mut self, value: ComponentValue) {
        self.value = value;
    }
}

/// https://www.w3.org/TR/css-syntax-3/#component-value
/// https://www.w3.org/TR/css-values-4/#component-types
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    /// https://www.w3.org/TR/css-values-3/#keywords
    Keyword(String),
    /// https://www.w3.org/TR/css-values-3/#numeric-types
    /// This is one of basic data types.
    Number(f64),
    /// https://www.w3.org/TR/css-syntax-3/#current-input-token
    /// The token from the list of tokens produced by the tokenizer.
    InputToken(CssToken),
}

#[derive(Debug, Clone)]
pub struct CssParser {
    t: std::iter::Peekable<CssTokenizer>,
}

impl CssParser {
    pub fn new(t: CssTokenizer) -> Self {
        Self { t: t.peekable() }
    }

    fn consume_ident(&mut self) -> String {
        let token = match self.t.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            CssToken::Ident(ref ident) => ident.to_string(),
            _ => {
                panic!("Parse error: {:?} is an unexpected token.", token);
            }
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-component-value
    fn consume_component_value(&mut self) -> ComponentValue {
        let token = match self.t.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            CssToken::Ident(ident) => ComponentValue::Keyword(ident.to_string()),
            CssToken::Number(num) => ComponentValue::Number(num.clone()),
            _ => ComponentValue::InputToken(token),
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#qualified-rule
    /// Note: Most qualified rules will be style rules, where the prelude is a selector [SELECT]
    /// and the block a list of declarations.
    fn consume_selector(&mut self) -> Selector {
        let token = match self.t.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            // TODO: support tag.class and tag#id
            CssToken::HashToken(value) => Selector::IdSelector(value[1..].to_string()),
            CssToken::Delim(delim) => {
                if delim == '.' {
                    return Selector::ClassSelector(self.consume_ident());
                }
                panic!("Parse error: {:?} is an unexpected token.", token);
            }
            CssToken::Ident(ident) => {
                // TODO: fix this. Skip pseudo-classes such as :link and :visited
                if self.t.peek() == Some(&CssToken::Colon) {
                    while self.t.peek() != Some(&CssToken::OpenCurly) {
                        self.t.next();
                    }
                }
                Selector::TypeSelector(ident.to_string())
            }
            CssToken::AtKeyword(_keyword) => {
                // skip until "{" comes
                while self.t.peek() != Some(&CssToken::OpenCurly) {
                    self.t.next();
                }
                Selector::UnknownSelector
            }
            _ => {
                panic!("warning: unexpected token {:?}", token);
            }
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-a-declaration
    fn consume_declaration(&mut self) -> Option<Declaration> {
        if self.t.peek().is_none() {
            return None;
        }

        // Create a new declaration with its name set to the value of the current input token.
        let mut declaration = Declaration::new();
        declaration.set_property(self.consume_ident());

        // "2. If the next input token is anything other than a <colon-token>, this is a parse error.
        // Return nothing. Otherwise, consume the next input token."
        match self.t.next() {
            Some(token) => match token {
                CssToken::Colon => {}
                _ => return None,
            },
            None => return None,
        }

        // "3. While the next input token is a <whitespace-token>, consume the next input token."

        // "4. As long as the next input token is anything other than an <EOF-token>, consume a
        // component value and append it to the declaration’s value."
        declaration.set_value(self.consume_component_value());

        Some(declaration)
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-simple-block
    /// https://www.w3.org/TR/css-syntax-3/#consume-a-list-of-declarations
    /// Note: Most qualified rules will be style rules, where the prelude is a selector [SELECT] and
    /// the block a list of declarations.
    fn consume_list_of_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return declarations,
            };

            match token {
                CssToken::CloseCurly => {
                    // https://www.w3.org/TR/css-syntax-3/#ending-token
                    assert_eq!(self.t.next(), Some(CssToken::CloseCurly));
                    return declarations;
                }
                CssToken::SemiColon => {
                    assert_eq!(self.t.next(), Some(CssToken::SemiColon));
                    // Do nothing.
                }
                CssToken::Ident(ref _ident) => match self.consume_declaration() {
                    Some(declaration) => {
                        declarations.push(declaration);
                        if self.t.peek() == Some(&CssToken::Delim(',')) {
                            self.t.next();
                        }
                    }
                    None => {}
                },
                CssToken::StringToken(_) => {
                    self.t.next();
                    if self.t.peek() == Some(&CssToken::Delim(',')) {
                        self.t.next();
                    }
                }
                CssToken::Number(_) => {
                    self.t.next();
                    if self.t.peek() == Some(&CssToken::Delim(',')) {
                        self.t.next();
                    }
                }
                _ => {
                    println!("warning: unexpected token {:?}", token);
                    self.t.next();
                }
            }
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-at-rule
    fn consume_at_rule(&mut self) -> Option<AtRule> {
        let rule = AtRule::new();

        loop {
            let token = match self.t.next() {
                Some(t) => t,
                None => return None,
            };

            match token {
                CssToken::OpenCurly => {
                    //TODO: set rule to AtRule.
                    let _qualified_rule = self.consume_qualified_rule();
                    // consume the close curly for a AtRule block
                    assert_eq!(self.t.next(), Some(CssToken::CloseCurly));
                    return Some(rule);
                }
                _ => {
                    println!("consume_at_rule anything else: {:?}", token);
                    // TODO: set prelude to AtRule
                }
            }
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-qualified-rule
    /// https://www.w3.org/TR/css-syntax-3/#qualified-rule
    /// https://www.w3.org/TR/css-syntax-3/#style-rules
    fn consume_qualified_rule(&mut self) -> Option<QualifiedRule> {
        let mut rule = QualifiedRule::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return None,
            };

            match token {
                CssToken::OpenCurly => {
                    // "Consume a simple block and assign it to the qualified rule’s block. Return
                    // the qualified rule."

                    // The content of the qualified rule’s block is parsed as a list of
                    // declarations.
                    assert_eq!(self.t.next(), Some(CssToken::OpenCurly));
                    rule.set_declarations(self.consume_list_of_declarations());
                    return Some(rule);
                }
                _ => {
                    // "Reconsume the current input token. Consume a component value. Append the
                    // returned value to the qualified rule’s prelude."

                    // The prelude of the qualified rule is parsed as a <selector-list>.
                    // https://www.w3.org/TR/css-syntax-3/#css-parse-something-according-to-a-css-grammar
                    rule.set_selector(self.consume_selector());
                }
            }
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-a-list-of-rules
    fn consume_list_of_rules(&mut self) -> Vec<QualifiedRule> {
        // "Create an initially empty list of rules."
        let mut rules = Vec::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return rules,
            };
            match token {
                // <at-keyword-token>
                // "Reconsume the current input token. Consume an at-rule, and append the returned value
                // to the list of rules."
                CssToken::AtKeyword(_keyword) => {
                    let _rule = self.consume_at_rule();
                    // TODO: we ignore media query for now. implement it properly.
                }
                _ => {
                    // anything else
                    // "Reconsume the current input token. Consume a qualified rule. If anything is
                    // returned, append it to the list of rules."
                    let rule = self.consume_qualified_rule();
                    match rule {
                        Some(r) => rules.push(r),
                        None => return rules,
                    }
                }
            }
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#parse-stylesheet
    pub fn parse_stylesheet(&mut self) -> StyleSheet {
        // 1. Create a new stylesheet.
        let mut sheet = StyleSheet::new();

        // 2. Consume a list of rules from the stream of tokens, with the top-level flag set. Let
        // the return value be rules.
        // 3. Assign rules to the stylesheet’s value.
        sheet.set_rules(self.consume_list_of_rules());

        // 4. Return the stylesheet.
        sheet
    }
}
