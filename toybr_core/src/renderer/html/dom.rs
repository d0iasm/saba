//! This is a part of "13.2.6 Tree construction" in the HTML spec.
//! https://html.spec.whatwg.org/multipage/parsing.html#tree-construction

use crate::browser::Browser;
use crate::common::ui::UiObject;
use crate::renderer::html::attribute::Attribute;
use crate::renderer::html::token::{HtmlToken, HtmlTokenizer, State};
use crate::utils::*;
use alloc::format;
use alloc::rc::{Rc, Weak};
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::{Display, Formatter};
use core::str::FromStr;

#[derive(Debug, Clone)]
/// https://dom.spec.whatwg.org/#interface-node
pub struct Node {
    kind: NodeKind,
    parent: Option<Weak<RefCell<Node>>>,
    first_child: Option<Rc<RefCell<Node>>>,
    last_child: Option<Weak<RefCell<Node>>>,
    previous_sibling: Option<Weak<RefCell<Node>>>,
    next_sibling: Option<Rc<RefCell<Node>>>,
}

///dom.spec.whatwg.org/#interface-node
impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
        }
    }

    pub fn kind(&self) -> NodeKind {
        self.kind.clone()
    }

    fn element_kind(&self) -> Option<ElementKind> {
        match self.kind {
            NodeKind::Document | NodeKind::Text(_) => None,
            NodeKind::Element(ref e) => Some(e.kind()),
        }
    }

    pub fn update_first_child(&mut self, first_child: Option<Rc<RefCell<Node>>>) {
        self.first_child = first_child;
    }

    pub fn first_child(&self) -> Option<Rc<RefCell<Node>>> {
        self.first_child.as_ref().map(|n| n.clone())
    }

    #[allow(dead_code)]
    pub fn last_child(&self) -> Option<Weak<RefCell<Node>>> {
        self.last_child.as_ref().map(|n| n.clone())
    }

    #[allow(dead_code)]
    pub fn previous_sibling(&self) -> Option<Weak<RefCell<Node>>> {
        self.previous_sibling.as_ref().map(|n| n.clone())
    }

    pub fn next_sibling(&self) -> Option<Rc<RefCell<Node>>> {
        self.next_sibling.as_ref().map(|n| n.clone())
    }
}

#[derive(Debug, Clone, Eq)]
pub enum NodeKind {
    /// https://dom.spec.whatwg.org/#interface-document
    Document,
    /// https://dom.spec.whatwg.org/#interface-element
    Element(Element),
    /// https://dom.spec.whatwg.org/#interface-text
    Text(String),
}

impl PartialEq for NodeKind {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            NodeKind::Document => match &other {
                NodeKind::Document => true,
                _ => false,
            },
            NodeKind::Element(e1) => match &other {
                NodeKind::Element(e2) => e1.kind == e2.kind,
                _ => false,
            },
            NodeKind::Text(_) => match &other {
                NodeKind::Text(_) => true,
                _ => false,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// https://dom.spec.whatwg.org/#interface-element
pub struct Element {
    kind: ElementKind,
    attributes: Vec<Attribute>,
}

impl Element {
    pub fn new(element_name: &str, attributes: Vec<Attribute>) -> Self {
        Self {
            kind: ElementKind::from_str(element_name)
                .expect("failed to convert string to ElementKind"),
            attributes,
        }
    }

    pub fn kind(&self) -> ElementKind {
        self.kind
    }

    pub fn attributes(&self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// https://dom.spec.whatwg.org/#interface-element
pub enum ElementKind {
    /// https://html.spec.whatwg.org/multipage/semantics.html#the-html-element
    Html,
    /// https://html.spec.whatwg.org/multipage/semantics.html#the-head-element
    Head,
    /// https://html.spec.whatwg.org/multipage/semantics.html#the-style-element
    Style,
    /// https://html.spec.whatwg.org/multipage/scripting.html#the-script-element
    Script,
    /// https://html.spec.whatwg.org/multipage/sections.html#the-body-element
    Body,
    /// https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements
    H1,
    H2,
    /// https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element
    P,
    /// https://html.spec.whatwg.org/multipage/grouping-content.html#the-pre-element
    Pre,
    /// https://html.spec.whatwg.org/multipage/grouping-content.html#the-ul-element
    Ul,
    /// https://html.spec.whatwg.org/multipage/grouping-content.html#the-li-element
    Li,
    /// https://html.spec.whatwg.org/multipage/grouping-content.html#the-div-element
    Div,
    /// https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element
    A,
}

impl ElementKind {
    pub fn to_string(&self) -> String {
        match self {
            ElementKind::Html => "html",
            ElementKind::Head => "head",
            ElementKind::Style => "style",
            ElementKind::Script => "script",
            ElementKind::Body => "body",
            ElementKind::H1 => "h1",
            ElementKind::H2 => "h2",
            ElementKind::P => "p",
            ElementKind::Pre => "pre",
            ElementKind::Ul => "ul",
            ElementKind::Li => "li",
            ElementKind::Div => "div",
            ElementKind::A => "a",
        }
        .to_string()
    }
}

impl Display for ElementKind {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        match self {
            ElementKind::Html => write!(f, "html"),
            ElementKind::Head => write!(f, "head"),
            ElementKind::Style => write!(f, "style"),
            ElementKind::Script => write!(f, "script"),
            ElementKind::Body => write!(f, "body"),
            ElementKind::H1 => write!(f, "h1"),
            ElementKind::H2 => write!(f, "h2"),
            ElementKind::P => write!(f, "p"),
            ElementKind::Pre => write!(f, "pre"),
            ElementKind::Ul => write!(f, "ul"),
            ElementKind::Li => write!(f, "li"),
            ElementKind::Div => write!(f, "div"),
            ElementKind::A => write!(f, "a"),
        }
    }
}

impl FromStr for ElementKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "html" => Ok(ElementKind::Html),
            "head" => Ok(ElementKind::Head),
            "style" => Ok(ElementKind::Style),
            "script" => Ok(ElementKind::Script),
            "body" => Ok(ElementKind::Body),
            "h1" => Ok(ElementKind::H1),
            "h2" => Ok(ElementKind::H2),
            "p" => Ok(ElementKind::P),
            "pre" => Ok(ElementKind::Pre),
            "ul" => Ok(ElementKind::Ul),
            "li" => Ok(ElementKind::Li),
            "div" => Ok(ElementKind::Div),
            "a" => Ok(ElementKind::A),
            _ => Err(format!("unimplemented element name {:?}", s)),
        }
    }
}

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
pub struct HtmlParser<U: UiObject> {
    browser: Weak<RefCell<Browser<U>>>,
    root: Rc<RefCell<Node>>,
    mode: InsertionMode,
    t: HtmlTokenizer,
    /// https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>,
    /// https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode
    original_insertion_mode: InsertionMode,
}

impl<U: UiObject> HtmlParser<U> {
    pub fn new(browser: Weak<RefCell<Browser<U>>>, t: HtmlTokenizer) -> Self {
        Self {
            browser,
            root: Rc::new(RefCell::new(Node::new(NodeKind::Document))),
            mode: InsertionMode::Initial,
            t,
            stack_of_open_elements: Vec::new(),
            original_insertion_mode: InsertionMode::Initial,
        }
    }

    /// Creates a char node.
    fn create_char(&self, c: char) -> Node {
        let mut s = String::new();
        s.push(c);
        return Node::new(NodeKind::Text(s));
    }

    /// Creates an element node.
    fn create_element(&self, tag: &str, attributes: Vec<Attribute>) -> Node {
        return Node::new(NodeKind::Element(Element::new(tag, attributes)));
    }

    /// Creates an element node for the token and insert it to the appropriate place for inserting
    /// a node. Put the new node in the stack of open elements.
    /// https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element
    fn insert_element(&mut self, tag: &str, attributes: Vec<Attribute>) {
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n,
            None => &self.root,
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

            last_sibiling.unwrap().borrow_mut().next_sibling = Some(node.clone());
            node.borrow_mut().previous_sibling =
                Some(Rc::downgrade(&current.borrow().first_child().unwrap()));
        } else {
            current.borrow_mut().first_child = Some(node.clone());
        }

        current.borrow_mut().last_child = Some(Rc::downgrade(&node));
        node.borrow_mut().parent = Some(Rc::downgrade(&current));

        self.stack_of_open_elements.push(node);
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character
    fn insert_char(&mut self, c: char) {
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n,
            None => &self.root,
        };

        // When the current node is Text, add a character to the current node.
        match current.borrow_mut().kind {
            NodeKind::Text(ref mut s) => {
                s.push(c);
                return;
            }
            _ => {}
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
                .next_sibling = Some(node.clone());
            node.borrow_mut().previous_sibling =
                Some(Rc::downgrade(&current.borrow().first_child().unwrap()));
        } else {
            current.borrow_mut().first_child = Some(node.clone());
        }

        current.borrow_mut().last_child = Some(Rc::downgrade(&node));
        node.borrow_mut().parent = Some(Rc::downgrade(&current));

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
        assert!(self.contain_in_stack(element_kind));

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

    pub fn construct_tree(&mut self) -> Rc<RefCell<Node>> {
        let mut token = self.t.next();

        while token.is_some() {
            match self.mode {
                // https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
                InsertionMode::Initial => self.mode = InsertionMode::BeforeHtml,

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
                        Some(HtmlToken::EndTag {
                            ref tag,
                            self_closing: _,
                        }) => {
                            // Any other end tag
                            // Parse error. Ignore the token.
                            if tag != "head" || tag != "body" || tag != "html" || tag != "br" {
                                // Ignore the token.
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.root.clone();
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
                            return self.root.clone();
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
                                self.insert_element(tag, attributes.to_vec());
                                self.original_insertion_mode = self.mode;
                                self.mode = InsertionMode::Text;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::EndTag {
                            ref tag,
                            self_closing: _,
                        }) => {
                            if tag == "head" {
                                self.mode = InsertionMode::AfterHead;
                                token = self.t.next();
                                self.pop_until(ElementKind::Head);
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.root.clone();
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
                            return self.root.clone();
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
                            self_closing: _,
                            ref attributes,
                        }) => {
                            match tag.as_str() {
                                // A start tag whose tag name is one of: "base", "basefont",
                                // "bgsound", "link", "meta", "noframes", "script", "style",
                                // "template", "title"
                                "script" => {
                                    // Process the token using the rules for the "in head" insertion mode.
                                    //
                                    // https://html.spec.whatwg.org/multipage/parsing.html#parsing-html-fragments
                                    // Switch the tokenizer to the script data state.
                                    self.t.switch_context(State::ScriptData);

                                    self.mode = InsertionMode::InHead;
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
                                _ => {
                                    console_warning(
                                        self.browser.clone(),
                                        format!("unknown tag {:?}", tag),
                                    );
                                    token = self.t.next();
                                }
                            }
                        }
                        Some(HtmlToken::EndTag {
                            ref tag,
                            self_closing: _,
                        }) => {
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
                                        self.browser.clone(),
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
                            return self.root.clone();
                        }
                    }
                } // end of InsertionMode::InBody

                // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incdata
                InsertionMode::Text => {
                    match token {
                        Some(HtmlToken::Eof) | None => {
                            return self.root.clone();
                        }
                        Some(HtmlToken::EndTag {
                            ref tag,
                            self_closing: _,
                        }) => {
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
                        Some(HtmlToken::EndTag {
                            ref tag,
                            self_closing: _,
                        }) => {
                            if tag == "html" {
                                self.mode = InsertionMode::AfterAfterBody;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.root.clone();
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
                        Some(HtmlToken::EndTag {
                            ref tag,
                            self_closing: _,
                        }) => {
                            if tag == "html" {
                                self.mode = InsertionMode::AfterAfterBody;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.root.clone();
                        }
                        _ => {}
                    }

                    self.mode = InsertionMode::InBody;
                } // end of InsertionMode::AfterAfterBody
            } // end of match self.mode {}
        } // end of while token.is_some {}

        self.root.clone()
    }
}

pub fn get_element_by_id(
    node: Option<Rc<RefCell<Node>>>,
    id_name: &String,
) -> Option<Rc<RefCell<Node>>> {
    match node {
        Some(n) => {
            match n.borrow().kind() {
                NodeKind::Element(e) => {
                    for attr in &e.attributes() {
                        if attr.name() == "id" && attr.value() == *id_name {
                            return Some(n.clone());
                        }
                    }
                }
                _ => {}
            }

            let result1 = get_element_by_id(n.borrow().first_child(), id_name);
            let result2 = get_element_by_id(n.borrow().next_sibling(), id_name);
            if result1.is_none() && result2.is_none() {
                return None;
            }
            if result1.is_none() {
                return result2;
            }

            return result1;
        }
        None => return None,
    }
}

fn get_target_element_node(
    node: Option<Rc<RefCell<Node>>>,
    element_kind: ElementKind,
) -> Option<Rc<RefCell<Node>>> {
    match node {
        Some(n) => {
            if n.borrow().kind
                == NodeKind::Element(Element::new(&element_kind.to_string(), Vec::new()))
            {
                return Some(n.clone());
            }
            let result1 = get_target_element_node(n.borrow().first_child(), element_kind);
            let result2 = get_target_element_node(n.borrow().next_sibling(), element_kind);
            if result1.is_none() && result2.is_none() {
                return None;
            }
            if result1.is_none() {
                return result2;
            }
            return result1;
        }
        None => return None,
    }
}

pub fn get_style_content(root: Rc<RefCell<Node>>) -> String {
    let style_node = match get_target_element_node(Some(root), ElementKind::Style) {
        Some(node) => node,
        None => return "".to_string(),
    };
    let text_node = match style_node.borrow().first_child() {
        Some(node) => node,
        None => return "".to_string(),
    };
    let content = match &text_node.borrow().kind {
        NodeKind::Text(ref s) => s.clone(),
        _ => "".to_string(),
    };
    content
}

pub fn get_js_content(root: Rc<RefCell<Node>>) -> String {
    let js_node = match get_target_element_node(Some(root), ElementKind::Script) {
        Some(node) => node,
        None => return "".to_string(),
    };
    let text_node = match js_node.borrow().first_child() {
        Some(node) => node,
        None => return "".to_string(),
    };
    let content = match &text_node.borrow().kind {
        NodeKind::Text(ref s) => s.clone(),
        _ => "".to_string(),
    };
    content
}
