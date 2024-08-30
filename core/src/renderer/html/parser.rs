//! This is a part of "13.2.6 Tree construction" in the HTML spec.
//! https://html.spec.whatwg.org/multipage/parsing.html#tree-construction

use crate::browser::Browser;
use crate::renderer::dom::node::Element;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::dom::window::Window;
use crate::renderer::html::attribute::Attribute;
use crate::renderer::html::token::{HtmlToken, HtmlTokenizer, State};
use crate::utils::console_warning;
use alloc::format;
use alloc::rc::{Rc, Weak};
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::str::FromStr;

/// https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    Text,
    AfterBody,
    AfterAfterBody,
}

#[derive(Debug, Clone)]
pub struct HtmlParser {
    browser: Weak<RefCell<Browser>>,
    window: Rc<RefCell<Window>>,
    mode: InsertionMode,
    /// https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode
    original_insertion_mode: InsertionMode,
    /// https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>,
    t: HtmlTokenizer,
}

impl HtmlParser {
    pub fn new(browser: Weak<RefCell<Browser>>, t: HtmlTokenizer) -> Self {
        Self {
            browser: browser.clone(),
            window: Rc::new(RefCell::new(Window::new(browser))),
            mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            stack_of_open_elements: Vec::new(),
            t,
        }
    }

    /// Creates a char node.
    fn create_char(&self, c: char) -> Node {
        let mut s = String::new();
        s.push(c);
        Node::new(NodeKind::Text(s))
    }

    /// Creates an element node.
    fn create_element(&self, tag: &str, attributes: Vec<Attribute>) -> Node {
        Node::new(NodeKind::Element(Element::new(tag, attributes)))
    }

    /// Creates an element node for the token and insert it to the appropriate place for inserting
    /// a node. Put the new node in the stack of open elements.
    /// https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element
    fn insert_element(&mut self, tag: &str, attributes: Vec<Attribute>) {
        let window = self.window.borrow();
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n.clone(),
            None => window.document(),
        };

        let node = Rc::new(RefCell::new(self.create_element(tag, attributes)));

        if current.borrow().first_child().is_some() {
            let mut last_sibiling = current.borrow().first_child();
            loop {
                last_sibiling = match last_sibiling {
                    Some(ref node) => {
                        if node.borrow().next_sibling().is_some() {
                            node.borrow().next_sibling()
                        } else {
                            break;
                        }
                    }
                    None => unimplemented!("last_sibiling should be Some"),
                };
            }

            last_sibiling
                .unwrap()
                .borrow_mut()
                .set_next_sibling(Some(node.clone()));
            node.borrow_mut().set_previous_sibling(Rc::downgrade(
                &current
                    .borrow()
                    .first_child()
                    .expect("failed to get a first child"),
            ))
        } else {
            current.borrow_mut().set_first_child(Some(node.clone()));
        }

        current.borrow_mut().set_last_child(Rc::downgrade(&node));
        node.borrow_mut().set_parent(Rc::downgrade(&current));

        self.stack_of_open_elements.push(node);
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character
    fn insert_char(&mut self, c: char) {
        let window = self.window.borrow();
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n.clone(),
            None => window.document(),
        };

        // When the current node is Text, add a character to the current node.
        // Do not access by current.borrow().kind(), otherwise, you can't add a next char to a
        // correct node.
        if let NodeKind::Text(ref mut s) = current.borrow_mut().kind {
            s.push(c);
            return;
        }

        // do not create a Text node if new char is '\n' or ' '
        if c == '\n' || c == ' ' {
            return;
        }

        let node = Rc::new(RefCell::new(self.create_char(c)));

        if current.borrow().first_child().is_some() {
            current
                .borrow()
                .first_child()
                .unwrap()
                .borrow_mut()
                .set_next_sibling(Some(node.clone()));
            node.borrow_mut().set_previous_sibling(Rc::downgrade(
                &current
                    .borrow()
                    .first_child()
                    .expect("failed to get a first child"),
            ));
        } else {
            current.borrow_mut().set_first_child(Some(node.clone()));
        }

        current.borrow_mut().set_last_child(Rc::downgrade(&node));
        node.borrow_mut().set_parent(Rc::downgrade(&current));

        self.stack_of_open_elements.push(node);
    }

    /// Returns true if the current node's kind is same as NodeKind::Element::<element_kind>.
    fn pop_current_node(&mut self, element_kind: ElementKind) -> bool {
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n,
            None => return false,
        };

        if current.borrow().element_kind() == Some(element_kind) {
            self.stack_of_open_elements.pop();
            return true;
        }

        false
    }

    /// Pops nodes until a node with `element_kind` comes.
    fn pop_until(&mut self, element_kind: ElementKind) {
        assert!(
            self.contain_in_stack(element_kind),
            "stack doesn't have an element {:?}",
            element_kind,
        );

        loop {
            let current = match self.stack_of_open_elements.pop() {
                Some(n) => n,
                None => return,
            };

            if current.borrow().element_kind() == Some(element_kind) {
                return;
            }
        }
    }

    /// Returns true if the stack of open elements has NodeKind::Element::<element_kind> node.
    fn contain_in_stack(&mut self, element_kind: ElementKind) -> bool {
        for i in 0..self.stack_of_open_elements.len() {
            if self.stack_of_open_elements[i].borrow().element_kind() == Some(element_kind) {
                return true;
            }
        }

        false
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#tree-construction
    pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
        let mut token = self.t.next();

        while token.is_some() {
            match self.mode {
                // https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
                InsertionMode::Initial => {
                    self.mode = InsertionMode::BeforeHtml;
                    continue;
                }

                // https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode
                InsertionMode::BeforeHtml => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            // A start tag whose tag name is "html"
                            // Create an element for the token in the HTML namespace, with the Document
                            // as the intended parent. Append it to the Document object. Put this
                            // element in the stack of open elements.
                            if tag == "html" {
                                self.insert_element(tag, attributes.to_vec());
                                self.mode = InsertionMode::BeforeHead;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            // Any other end tag
                            // Parse error. Ignore the token.
                            if tag != "head" || tag != "body" || tag != "html" || tag != "br" {
                                // Ignore the token.
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                    }
                    token = self.t.next();
                    //self.insert_element("html", Vec::new());
                    //self.mode = InsertionMode::BeforeHead;
                } // end of InsertionMode::BeforeHtml

                // https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode
                InsertionMode::BeforeHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            if tag == "head" {
                                self.insert_element(tag, attributes.to_vec());
                                self.mode = InsertionMode::InHead;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    token = self.t.next();
                    //self.insert_element("head", Vec::new());
                    //self.mode = InsertionMode::InHead;
                } // end of InsertionMode::BeforeHead

                // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead
                InsertionMode::InHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                self.insert_char(c);
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            if tag == "style" {
                                self.insert_element(tag, attributes.to_vec());
                                self.original_insertion_mode = self.mode;
                                self.mode = InsertionMode::Text;
                                token = self.t.next();
                                continue;
                            }
                            if tag == "script" {
                                // "6. Insert the newly created element at the adjusted insertion
                                // location."
                                // "7. Push the element onto the stack of open elements so that it is
                                // the new current node."
                                self.insert_element(tag, attributes.to_vec());
                                // "8. Switch the tokenizer to the script data state."
                                self.t.switch_context(State::ScriptData);
                                // "9. Let the original insertion mode be the current insertion mode."
                                self.original_insertion_mode = self.mode;
                                // "10. Switch the insertion mode to "text"."
                                self.mode = InsertionMode::Text;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag == "head" {
                                self.mode = InsertionMode::AfterHead;
                                token = self.t.next();
                                self.pop_until(ElementKind::Head);
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                    }
                    token = self.t.next();
                    //self.mode = InsertionMode::AfterHead;
                    //self.pop_until(ElementKind::Head);
                } // end of InsertionMode::InHead

                // https://html.spec.whatwg.org/multipage/parsing.html#the-after-head-insertion-mode
                InsertionMode::AfterHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                self.insert_char(c);
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            if tag == "body" {
                                self.insert_element(tag, attributes.to_vec());
                                token = self.t.next();
                                self.mode = InsertionMode::InBody;
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    token = self.t.next();
                    //self.insert_element("body", Vec::new());
                    //self.mode = InsertionMode::InBody;
                } // end of InsertionMode::AfterHead

                // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody
                InsertionMode::InBody => {
                    match token {
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing,
                            ref attributes,
                        }) => {
                            match tag.as_str() {
                                // A start tag whose tag name is one of: "base", "basefont",
                                // "bgsound", "link", "meta", "noframes", "script", "style",
                                // "template", "title"
                                "script" | "style" => {
                                    // Process the token using the rules for the "in head" insertion mode.
                                    //
                                    // https://html.spec.whatwg.org/multipage/parsing.html#parsing-html-fragments
                                    // Switch the tokenizer to the script data state.
                                    self.insert_element(tag, attributes.to_vec());
                                    self.t.switch_context(State::ScriptData);
                                    self.original_insertion_mode = self.mode;
                                    self.mode = InsertionMode::Text;
                                    token = self.t.next();
                                    continue;
                                }
                                // A start tag whose tag name is one of: "address", "article",
                                // "aside", "blockquote", "center", "details", "dialog", "dir",
                                // "div", "dl", "fieldset", "figcaption", "figure", "footer",
                                // "header", "hgroup", "main", "menu", "nav", "ol", "p", "section",
                                // "summary", "ul"
                                "div" | "p" | "ul" => {
                                    // If the stack of open elements has a p element in button
                                    // scope, then close a p element.
                                    //
                                    // Insert an HTML element for the token.
                                    self.insert_element(tag, attributes.to_vec());
                                    token = self.t.next();
                                    continue;
                                }
                                // A start tag whose tag name is one of: "h1", "h2", "h3", "h4",
                                // "h5", "h6"
                                "h1" | "h2" => {
                                    // If the stack of open elements has a p element in button
                                    // scope, then close a p element.
                                    //
                                    // If the current node is an HTML element whose tag name is one
                                    // of "h1", "h2", "h3", "h4", "h5", or "h6", then this is a
                                    // parse error; pop the current node off the stack of open
                                    // elements.
                                    //
                                    // Insert an HTML element for the token.
                                    self.insert_element(tag, attributes.to_vec());
                                    token = self.t.next();
                                    continue;
                                }
                                // A start tag whose tag name is one of: "pre", "listing"
                                "pre" => {
                                    // If the stack of open elements has a p element in button
                                    // scope, then close a p element.
                                    //
                                    // Insert an HTML element for the token.
                                    //
                                    // If the next token is a U+000A LINE FEED (LF) character
                                    // token, then ignore that token and move on to the next one.
                                    // (Newlines at the start of pre blocks are ignored as an
                                    // authoring convenience.)
                                    //
                                    // Set the frameset-ok flag to "not ok".
                                    self.insert_element(tag, attributes.to_vec());
                                    token = self.t.next();
                                    continue;
                                }
                                // A start tag whose tag name is "li"
                                "li" => {
                                    // Run these steps:
                                    //
                                    // 1. Set the frameset-ok flag to "not ok".
                                    //
                                    // 2. Initialize node to be the current node (the bottommost node
                                    // of the stack).
                                    //
                                    // 3. Loop: If node is an li element, then run these substeps:
                                    // 3-1. Generate implied end tags, except for li elements.
                                    // 3-2. If the current node is not an li element, then this is a
                                    // parse error.
                                    // 3-3. Pop elements from the stack of open elements until an li
                                    // element has been popped from the stack.
                                    // 3-4. Jump to the step labeled done below.
                                    //
                                    // 4. If node is in the special category, but is not an address,
                                    // div, or p element, then jump to the step labeled done below.
                                    //
                                    // 5. Otherwise, set node to the previous entry in the stack of
                                    // open elements and return to the step labeled loop.
                                    //
                                    // 6. Done: If the stack of open elements has a p element in
                                    // button scope, then close a p element.
                                    //
                                    // 7. Finally, insert an HTML element for the token.
                                    self.insert_element(tag, attributes.to_vec());
                                    token = self.t.next();
                                    continue;
                                }
                                // A start tag whose tag name is "a"
                                "a" => {
                                    // If the list of active formatting elements contains an a
                                    // element between the end of the list and the last marker on
                                    // the list (or the start of the list if there is no marker on
                                    // the list), then this is a parse error; run the adoption
                                    // agency algorithm for the token, then remove that element
                                    // from the list of active formatting elements and the stack of
                                    // open elements if the adoption agency algorithm didn't
                                    // already remove it (it might not have if the element is not
                                    // in table scope).
                                    //
                                    // Reconstruct the active formatting elements, if any.
                                    //
                                    // Insert an HTML element for the token. Push onto the list of
                                    // active formatting elements that element.
                                    self.insert_element(tag, attributes.to_vec());
                                    token = self.t.next();
                                    continue;
                                }
                                // A start tag whose tag name is one of: "area", "br", "embed", "img", "keygen", "wbr"
                                "img" => {
                                    // Reconstruct the active formatting elements, if any.

                                    // Insert an HTML element for the token. Immediately pop the current node off the stack of open elements.

                                    // Acknowledge the token's self-closing flag, if it is set.

                                    // Set the frameset-ok flag to "not ok".

                                    self.insert_element(tag, attributes.to_vec());
                                    if self_closing {
                                        self.stack_of_open_elements.pop();
                                    }
                                    token = self.t.next();
                                    continue;
                                }
                                _ => {
                                    console_warning(
                                        &self.browser,
                                        format!("unknown tag {:?}", tag),
                                    );
                                    token = self.t.next();
                                }
                            }
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            match tag.as_str() {
                                // An end tag whose tag name is "body"
                                "body" => {
                                    self.mode = InsertionMode::AfterBody;
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    if !self.contain_in_stack(ElementKind::Body) {
                                        // Parse error. Ignore the token.
                                        continue;
                                    }
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                // An end tag whose tag name is "html"
                                "html" => {
                                    // If the stack of open elements does not have a body element in
                                    // scope, this is a parse error; ignore the token.
                                    if self.pop_current_node(ElementKind::Body) {
                                        self.mode = InsertionMode::AfterBody;
                                        assert!(self.pop_current_node(ElementKind::Html));
                                    } else {
                                        token = self.t.next();
                                    }
                                    continue;
                                }
                                // An end tag whose tag name is one of: "address", "article",
                                // "aside", "blockquote", "button", "center", "details", "dialog",
                                // "dir", "div", "dl", "fieldset", "figcaption", "figure",
                                // "footer", "header", "hgroup", "listing", "main", "menu", "nav",
                                // "ol", "pre", "section", "summary", "ul"
                                "div" | "pre" | "ul" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                // An end tag whose tag name is "p"
                                "p" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                // An end tag whose tag name is "li"
                                "li" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                // An end tag whose tag name is one of: "h1", "h2", "h3", "h4",
                                // "h5", "h6"
                                "h1" | "h2" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                // An end tag whose tag name is one of: "a", "b", "big", "code",
                                // "em", "font", "i", "nobr", "s", "small", "strike", "strong",
                                // "tt", "u"
                                "a" => {
                                    // Run the adoption agency algorithm for the token.
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    token = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                _ => {
                                    console_warning(
                                        &self.browser,
                                        format!("unknown tag {:?}", tag),
                                    );
                                    token = self.t.next();
                                }
                            }
                        }
                        // Any other character token
                        Some(HtmlToken::Char(c)) => {
                            // TODO: Reconstruct the active formatting elements, if any.
                            // Insert the token's character.
                            // TODO: Set the frameset-ok flag to "not ok".
                            self.insert_char(c);
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                    }
                } // end of InsertionMode::InBody

                // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incdata
                InsertionMode::Text => {
                    match token {
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag == "style" {
                                self.pop_until(ElementKind::Style);
                                self.mode = self.original_insertion_mode;
                                token = self.t.next();
                                continue;
                            }
                            if tag == "script" {
                                self.pop_until(ElementKind::Script);
                                self.mode = self.original_insertion_mode;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Char(c)) => {
                            self.insert_char(c);
                            token = self.t.next();
                            continue;
                        }
                        _ => {}
                    }

                    self.mode = self.original_insertion_mode;
                } // end of InsertionMode::Text

                // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterbody
                InsertionMode::AfterBody => {
                    match token {
                        Some(HtmlToken::Char(_c)) => {
                            // Not align with the spec.
                            // TODO: Process the token using the rules for the "in body" insertion
                            // mode.
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag == "html" {
                                self.mode = InsertionMode::AfterAfterBody;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    self.mode = InsertionMode::InBody;
                } // end of InsertionMode::AfterBody

                // https://html.spec.whatwg.org/multipage/parsing.html#the-after-after-body-insertion-mode
                InsertionMode::AfterAfterBody => {
                    match token {
                        Some(HtmlToken::Char(_c)) => {
                            // Not align with the spec.
                            // TODO: Process the token using the rules for the "in body" insertion
                            // mode.
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    self.mode = InsertionMode::InBody;
                } // end of InsertionMode::AfterAfterBody
            } // end of match self.mode {}
        } // end of while token.is_some {}

        self.window.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_empty() {
        let browser = Browser::new();
        let html = "".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(Rc::downgrade(&browser), t).construct_tree();
        let expected = Rc::new(RefCell::new(Node::new(NodeKind::Document)));

        assert_eq!(expected, window.borrow().document());
    }

    #[test]
    fn test_body() {
        let browser = Browser::new();
        let html = "<html><head></head><body></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(Rc::downgrade(&browser), t).construct_tree();
        let document = window.borrow().document();
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Document))),
            document
        );
        let html = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "html",
                Vec::new()
            ))))),
            html
        );
        let head = html
            .borrow()
            .first_child()
            .expect("failed to get a first child of html");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "head",
                Vec::new()
            ))))),
            head
        );
        let body = head
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body
        );
    }

    #[test]
    fn test_text() {
        let browser = Browser::new();
        let html = "<html><head></head><body>text</body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(Rc::downgrade(&browser), t).construct_tree();
        let document = window.borrow().document();

        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Document))),
            document
        );
        let html = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "html",
                Vec::new()
            ))))),
            html
        );
        let body = html
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body
        );
        let text = body
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Text("text".to_string())))),
            text
        );
    }

    #[test]
    fn test_multiple_nodes() {
        let browser = Browser::new();
        let html = "<html><head></head><body><p><a foo=bar>text</a></p></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(Rc::downgrade(&browser), t).construct_tree();
        let document = window.borrow().document();

        let body = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body
        );

        let p = body
            .borrow()
            .first_child()
            .expect("failed to get a first child of body");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "p",
                Vec::new()
            ))))),
            p
        );

        let mut attr = Attribute::new();
        attr.add_char('f', true);
        attr.add_char('o', true);
        attr.add_char('o', true);
        attr.add_char('b', false);
        attr.add_char('a', false);
        attr.add_char('r', false);
        let a = p
            .borrow()
            .first_child()
            .expect("failed to get a first child of p");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "a",
                vec![attr]
            ))))),
            a
        );

        let text = a
            .borrow()
            .first_child()
            .expect("failed to get a first child of a");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Text("text".to_string())))),
            text
        );
    }
}
