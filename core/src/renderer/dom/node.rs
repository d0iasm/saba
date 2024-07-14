//! This is a DOM node.
//! https://dom.spec.whatwg.org/#interface-node
//!
//! https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/dom/node.h

use crate::renderer::dom::activation_behavior::get_activation_behavior;
use crate::renderer::dom::activation_behavior::ActivationBehavior;
use crate::renderer::dom::event::Event;
use crate::renderer::dom::event::EventListener;
use crate::renderer::dom::event::EventListenerCallback;
use crate::renderer::dom::event::EventTarget;
use crate::renderer::dom::window::Window;
use crate::renderer::html::attribute::Attribute;
use alloc::format;
use alloc::rc::{Rc, Weak};
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::{Display, Formatter};
use core::str::FromStr;

#[derive(Debug, Clone)]
/// https://dom.spec.whatwg.org/#interface-node
pub struct Node {
    pub kind: NodeKind,
    window: Weak<RefCell<Window>>,
    // TODO: add setter functions for other nodes.
    pub parent: Weak<RefCell<Node>>,
    pub first_child: Option<Rc<RefCell<Node>>>,
    pub last_child: Weak<RefCell<Node>>,
    pub previous_sibling: Weak<RefCell<Node>>,
    pub next_sibling: Option<Rc<RefCell<Node>>>,
    /// https://dom.spec.whatwg.org/#eventtarget-event-listener-list
    events: Vec<EventListener>,
    /// https://dom.spec.whatwg.org/#eventtarget-activation-behavior
    activation_behavior: Option<ActivationBehavior>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
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

    pub fn set_window(&mut self, window: Weak<RefCell<Window>>) {
        self.window = window;
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
            if attr.name() == name {
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
