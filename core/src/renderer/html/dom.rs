//! This is a part of "13.2.6 Tree construction" in the HTML spec.
//! https://html.spec.whatwg.org/multipage/parsing.html#tree-construction

use crate::browser::Browser;
use crate::renderer::dom::activation_behavior::get_activation_behavior;
use crate::renderer::dom::activation_behavior::ActivationBehavior;
use crate::renderer::dom::event::Event;
use crate::renderer::dom::event::EventListener;
use crate::renderer::dom::event::EventListenerCallback;
use crate::renderer::dom::event::EventTarget;
use crate::renderer::html::attribute::Attribute;
use crate::renderer::html::token::{HtmlToken, HtmlTokenizer, State};
use crate::renderer::page::Page;
use crate::utils::*;
use alloc::format;
use alloc::rc::{Rc, Weak};
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::{Display, Formatter};
use core::str::FromStr;

#[derive(Debug, Clone)]
/// https://html.spec.whatwg.org/multipage/nav-history-apis.html#window
pub struct Window {
    _browser: Weak<RefCell<Browser>>,
    page: Weak<RefCell<Page>>,
    document: Rc<RefCell<Node>>,
}

impl Window {
    pub fn new(browser: Weak<RefCell<Browser>>) -> Self {
        let window = Self {
            _browser: browser,
            page: Weak::new(),
            document: Rc::new(RefCell::new(Node::new(NodeKind::Document))),
        };

        window.document.borrow_mut().window = Rc::downgrade(&Rc::new(RefCell::new(window.clone())));

        window
    }

    pub fn document(&self) -> Rc<RefCell<Node>> {
        self.document.clone()
    }
}

#[derive(Debug, Clone)]
/// https://dom.spec.whatwg.org/#interface-node
pub struct Node {
    kind: NodeKind,
    window: Weak<RefCell<Window>>,
    parent: Weak<RefCell<Node>>,
    first_child: Option<Rc<RefCell<Node>>>,
    last_child: Weak<RefCell<Node>>,
    previous_sibling: Weak<RefCell<Node>>,
    next_sibling: Option<Rc<RefCell<Node>>>,
    /// https://dom.spec.whatwg.org/#eventtarget-event-listener-list
    events: Vec<EventListener>,
    /// https://dom.spec.whatwg.org/#eventtarget-activation-behavior
    activation_behavior: Option<ActivationBehavior>,
}

///dom.spec.whatwg.org/#interface-node
impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind: kind.clone(),
            window: Weak::new(),
            parent: Weak::new(),
            first_child: None,
            last_child: Weak::new(),
            previous_sibling: Weak::new(),
            next_sibling: None,
            events: Vec::new(),
            activation_behavior: get_activation_behavior(&kind),
        }
    }

    pub fn kind(&self) -> NodeKind {
        self.kind.clone()
    }

    pub fn get_window(&self) -> Weak<RefCell<Window>> {
        if self.window.upgrade().is_some() {
            return self.window.clone();
        }

        let mut current = match self.previous_sibling().upgrade() {
            Some(n) => n,
            None => match self.parent().upgrade() {
                Some(n) => n,
                None => panic!("either sibling or parent should exist"),
            },
        };

        loop {
            if current.borrow().window.upgrade().is_some() {
                return current.borrow().window.clone();
            }

            current = match current.clone().borrow().parent().upgrade() {
                Some(n) => n,
                None => panic!("parent should exist"),
            };
        }
    }

    pub fn get_element(&self) -> Option<Element> {
        match self.kind {
            NodeKind::Document | NodeKind::Text(_) => None,
            NodeKind::Element(ref e) => Some(e.clone()),
        }
    }

    pub fn element_kind(&self) -> Option<ElementKind> {
        match self.kind {
            NodeKind::Document | NodeKind::Text(_) => None,
            NodeKind::Element(ref e) => Some(e.kind()),
        }
    }

    pub fn update_first_child(&mut self, first_child: Option<Rc<RefCell<Node>>>) {
        self.first_child = first_child;
    }

    pub fn parent(&self) -> Weak<RefCell<Node>> {
        self.parent.clone()
    }

    pub fn first_child(&self) -> Option<Rc<RefCell<Node>>> {
        self.first_child.as_ref().cloned()
    }

    pub fn previous_sibling(&self) -> Weak<RefCell<Node>> {
        self.previous_sibling.clone()
    }

    pub fn next_sibling(&self) -> Option<Rc<RefCell<Node>>> {
        self.next_sibling.as_ref().cloned()
    }
}

/// https://dom.spec.whatwg.org/#interface-eventtarget
impl EventTarget for Node {
    fn target_kind(&self) -> NodeKind {
        self.kind()
    }

    /// https://dom.spec.whatwg.org/#dom-eventtarget-addeventlistener
    fn add_event_listener(&mut self, event_type: String, callback: EventListenerCallback) {
        for e in &self.events {
            if e.event_type() == event_type {
                // Do not add a new EventListener if the same event type already exists.
                return;
            }
        }
        self.events
            .push(EventListener::new(event_type, callback, false));
    }

    /// https://dom.spec.whatwg.org/#dom-eventtarget-removeeventlistener
    fn remove_event_listener(&mut self, event_type: String, _callback: EventListenerCallback) {
        if let Some(index) = self
            .events
            .iter()
            .position(|e| e.event_type() == event_type)
        {
            self.events.remove(index);
        }
    }

    /// https://dom.spec.whatwg.org/#dom-eventtarget-dispatchevent
    fn dispatch_event(&mut self, event: Event) -> bool {
        // https://dom.spec.whatwg.org/#concept-event-dispatch
        let mut activation_target: Option<Self> = None;
        match &event {
            // "5.4. Let isActivationEvent be true, if event is a MouseEvent object and event’s
            // type attribute is "click"; otherwise false."
            Event::MouseEvent(mouse_event) => {
                // "5. If target is not relatedTarget or target is event’s relatedTarget, then:"
                //
                // "5.5. If isActivationEvent is true and target has activation behavior, then set
                // activationTarget to target."
                if self.target_kind() == mouse_event.target.target_kind()
                    && mouse_event.event_type() == "click"
                {
                    activation_target = Some(self.clone());
                }
            }
        }

        // "11. If activationTarget is non-null, then:"
        if let Some(target) = activation_target {
            if let Some(activation_behavior) = target.activation_behavior {
                // "11.1. If event’s canceled flag is unset, then run activationTarget’s activation behavior
                // with event."
                // "11.2. Otherwise, if activationTarget has legacy-canceled-activation behavior, then run
                // activationTarget’s legacy-canceled-activation behavior."
                activation_behavior(Rc::new(RefCell::new(self.clone())), event);
            }
        }
        true
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
            NodeKind::Document => matches!(other, NodeKind::Document),
            NodeKind::Element(e1) => match &other {
                NodeKind::Element(e2) => e1.kind == e2.kind,
                _ => false,
            },
            NodeKind::Text(_) => matches!(other, NodeKind::Text(_)),
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

    /// Returns a value for an attribute `name`.
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        for attr in &self.attributes {
            if &attr.name() == name {
                return Some(attr.value());
            }
        }
        None
    }

    /// return true if this element is a block element
    pub fn is_block_element(&self) -> bool {
        match self.kind {
            // https://developer.mozilla.org/en-US/docs/Web/HTML/Block-level_elements#elements
            ElementKind::H1
            | ElementKind::H2
            | ElementKind::P
            | ElementKind::Pre
            | ElementKind::Ul
            | ElementKind::Li
            | ElementKind::Div => true,
            // https://developer.mozilla.org/en-US/docs/Web/HTML/Inline_elements#list_of_inline_elements
            _ => false,
        }
    }

    /*
    /// https://html.spec.whatwg.org/multipage/dom.html#flow-content-2
    /// return true if this element should exist inside a body element
    pub fn is_flow_content(&self) -> bool {
        match self.kind {
            // https://html.spec.whatwg.org/multipage/scripting.html#the-script-element
            // ElementKind::Script should be a flow content
            ElementKind::Body
            | ElementKind::H1
            | ElementKind::H2
            | ElementKind::P
            | ElementKind::Pre
            | ElementKind::Ul
            | ElementKind::Li
            | ElementKind::Div
            | ElementKind::A => true,
            _ => false,
        }
    }
    */
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
    /// https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element
    IMG,
}

impl Display for ElementKind {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let s = match self {
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
            ElementKind::IMG => "img",
        };
        write!(f, "{}", s)
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
            "img" => Ok(ElementKind::IMG),
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
pub struct HtmlParser {
    browser: Weak<RefCell<Browser>>,
    window: Rc<RefCell<Window>>,
    mode: InsertionMode,
    t: HtmlTokenizer,
    /// https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>,
    /// https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode
    original_insertion_mode: InsertionMode,
}

impl HtmlParser {
    pub fn new(browser: Weak<RefCell<Browser>>, t: HtmlTokenizer) -> Self {
        Self {
            browser: browser.clone(),
            window: Rc::new(RefCell::new(Window::new(browser))),
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
            Some(n) => n,
            None => &window.document,
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
                Rc::downgrade(&current.borrow().first_child().unwrap());
        } else {
            current.borrow_mut().first_child = Some(node.clone());
        }

        current.borrow_mut().last_child = Rc::downgrade(&node);
        node.borrow_mut().parent = Rc::downgrade(current);

        self.stack_of_open_elements.push(node);
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character
    fn insert_char(&mut self, c: char) {
        let window = self.window.borrow();
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n,
            None => &window.document,
        };

        // When the current node is Text, add a character to the current node.
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
                .next_sibling = Some(node.clone());
            node.borrow_mut().previous_sibling =
                Rc::downgrade(&current.borrow().first_child().unwrap());
        } else {
            current.borrow_mut().first_child = Some(node.clone());
        }

        current.borrow_mut().last_child = Rc::downgrade(&node);
        node.borrow_mut().parent = Rc::downgrade(current);

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

    /// https://html.spec.whatwg.org/multipage/parsing.html#tree-construction
    pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
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
                                // A start tag whose tag name is one of: "area", "br", "embed", "img", "keygen", "wbr"
                                "img" => {
                                    // Reconstruct the active formatting elements, if any.

                                    // Insert an HTML element for the token. Immediately pop the current node off the stack of open elements.

                                    // Acknowledge the token's self-closing flag, if it is set.

                                    // Set the frameset-ok flag to "not ok".

                                    // TODO: handle self-closing flag.
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
