//! https://www.w3.org/TR/cssom/
//!
//! This is a part of "3. Tokenizing and Parsing CSS" in the "CSS Syntax Module Level 3" spec.
//! https://www.w3.org/TR/css-syntax-3/#tokenizing-and-parsing
//!
//! 5. Parsing
//! https://www.w3.org/TR/css-syntax-3/#parsing

use crate::browser::Browser;
use crate::renderer::css::token::CssToken;
use crate::renderer::css::token::CssTokenizer;
use crate::utils::console_warning;
use alloc::format;
use alloc::rc::Weak;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::iter::Peekable;

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

/// https://www.w3.org/TR/cssom-1/#cssstylesheet
/// https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleRule
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheet {
    /// https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssrules
    pub rules: Vec<QualifiedRule>,
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self::new()
    }
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

impl Default for AtRule {
    fn default() -> Self {
        Self::new()
    }
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
/// https://www.w3.org/TR/cssom-1/#cssstylerule
/// https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleRule
pub struct QualifiedRule {
    // TODO: support multiple selectors
    /// https://www.w3.org/TR/selectors-4/#typedef-selector-list
    /// The prelude of the qualified rule is parsed as a <selector-list>.
    pub selector: Selector,
    /// https://www.w3.org/TR/css-syntax-3/#parse-a-list-of-declarations
    /// The content of the qualified rule’s block is parsed as a list of declarations.
    pub declarations: Vec<Declaration>,
}

impl Default for QualifiedRule {
    fn default() -> Self {
        Self::new()
    }
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

/// https://www.w3.org/TR/selectors-4/
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// https://www.w3.org/TR/selectors-4/#type-selectors
    TypeSelector(String),
    /// https://www.w3.org/TR/selectors-4/#class-html
    ClassSelector(String),
    /// https://www.w3.org/TR/selectors-4/#id-selectors
    IdSelector(String),
    /// This is an unofficial selector.
    UnknownSelector,
}

#[derive(Debug, Clone, PartialEq)]
/// https://www.w3.org/TR/css-syntax-3/#declaration
/// https://www.w3.org/TR/cssom-1/#the-cssstyledeclaration-interface
/// https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleDeclaration
pub struct Declaration {
    pub property: String,
    pub value: ComponentValue,
}

impl Default for Declaration {
    fn default() -> Self {
        Self::new()
    }
}

impl Declaration {
    pub fn new() -> Self {
        Self {
            property: String::new(),
            value: ComponentValue::Ident(String::new()),
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
pub type ComponentValue = CssToken;

#[derive(Debug, Clone)]
pub struct CssParser {
    browser: Weak<RefCell<Browser>>,
    t: Peekable<CssTokenizer>,
}

impl CssParser {
    pub fn new(browser: Weak<RefCell<Browser>>, t: CssTokenizer) -> Self {
        Self {
            browser,
            t: t.peekable(),
        }
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
        self.t
            .next()
            .expect("should have a token in consume_component_value")
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
                console_warning(&self.browser, format!("unexpected token {:?}", token));
                self.t.next();
                Selector::UnknownSelector
            }
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-a-declaration
    fn consume_declaration(&mut self) -> Option<Declaration> {
        // Create a new declaration with its name set to the value of the current input token.
        let mut declaration = Declaration::new();
        declaration.set_property(self.consume_ident());

        // "2. If the next input token is anything other than a <colon-token>, this is a parse error.
        // Return nothing. Otherwise, consume the next input token."
        match self.t.next() {
            Some(CssToken::Colon) => {}
            _ => return None,
        }

        // "3. While the next input token is a <whitespace-token>, consume the next input token."

        // "4. As long as the next input token is anything other than an <EOF-token>, consume a
        // component value and append it to the declaration’s value."
        // TODO: support multiple values in one declaration.
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
                CssToken::Ident(ref _ident) => {
                    if let Some(declaration) = self.consume_declaration() {
                        declarations.push(declaration);
                    }
                }
                _ => {
                    console_warning(
                        &self.browser,
                        format!(
                            "unexpected token in consume_list_of_declarations {:?}",
                            token
                        ),
                    );
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
                    console_warning(
                        &self.browser,
                        format!("consume_at_rule anything else: {:?}", token),
                    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use alloc::vec;

    #[test]
    fn test_empty() {
        let browser = Browser::new();
        let style = "".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(Rc::downgrade(&browser), t).parse_stylesheet();

        assert_eq!(cssom.rules.len(), 0);
    }

    #[test]
    fn test_one_rule() {
        let browser = Browser::new();
        let style = "p { color: red; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(Rc::downgrade(&browser), t).parse_stylesheet();

        let mut rule = QualifiedRule::default();
        rule.set_selector(Selector::TypeSelector("p".to_string()));
        let mut declaration = Declaration::default();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("red".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        let mut i = 0;
        for rule in &cssom.rules {
            assert_eq!(&expected[i], rule);
            i += 1;
        }
    }

    #[test]
    fn test_id_selector() {
        let browser = Browser::new();
        let style = "#id { color: red; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(Rc::downgrade(&browser), t).parse_stylesheet();

        let mut rule = QualifiedRule::default();
        rule.set_selector(Selector::IdSelector("id".to_string()));
        let mut declaration = Declaration::default();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("red".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        let mut i = 0;
        for rule in &cssom.rules {
            assert_eq!(&expected[i], rule);
            i += 1;
        }
    }

    #[test]
    fn test_class_selector() {
        let browser = Browser::new();
        let style = ".class { color: red; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(Rc::downgrade(&browser), t).parse_stylesheet();

        let mut rule = QualifiedRule::default();
        rule.set_selector(Selector::ClassSelector("class".to_string()));
        let mut declaration = Declaration::default();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("red".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        let mut i = 0;
        for rule in &cssom.rules {
            assert_eq!(&expected[i], rule);
            i += 1;
        }
    }

    #[test]
    fn test_multiple_rules() {
        let browser = Browser::new();
        let style = "p { content: \"Hey\"; } h1 { font-size: 40; color: blue; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(Rc::downgrade(&browser), t).parse_stylesheet();

        let mut rule1 = QualifiedRule::default();
        rule1.set_selector(Selector::TypeSelector("p".to_string()));
        let mut declaration1 = Declaration::default();
        declaration1.set_property("content".to_string());
        declaration1.set_value(ComponentValue::StringToken("Hey".to_string()));
        rule1.set_declarations(vec![declaration1]);

        let mut rule2 = QualifiedRule::default();
        rule2.set_selector(Selector::TypeSelector("h1".to_string()));
        let mut declaration2 = Declaration::default();
        declaration2.set_property("font-size".to_string());
        declaration2.set_value(ComponentValue::Number(40.0));
        let mut declaration3 = Declaration::default();
        declaration3.set_property("color".to_string());
        declaration3.set_value(ComponentValue::Ident("blue".to_string()));
        rule2.set_declarations(vec![declaration2, declaration3]);

        let expected = [rule1, rule2];
        assert_eq!(cssom.rules.len(), expected.len());

        let mut i = 0;
        for rule in &cssom.rules {
            assert_eq!(&expected[i], rule);
            i += 1;
        }
    }
}
