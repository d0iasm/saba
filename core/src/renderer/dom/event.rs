//! This is a part of "UI Events" in the W3C specification.
//! DOM Living Standard: https://dom.spec.whatwg.org/#events
//! UI Events W3C Working Draft: https://www.w3.org/TR/uievents/

use crate::renderer::html::parser::NodeKind;
use alloc::boxed::Box;
use alloc::string::String;

/// https://dom.spec.whatwg.org/#callbackdef-eventlistener
pub type EventListenerCallback = fn(e: Event);

/// https://dom.spec.whatwg.org/#concept-event-listener
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EventListener {
    event_type: String,
    callback: EventListenerCallback,
    capture: bool,
}

impl EventListener {
    pub fn new(event_type: String, callback: EventListenerCallback, capture: bool) -> Self {
        Self {
            event_type,
            callback,
            capture,
        }
    }

    pub fn event_type(&self) -> String {
        self.event_type.clone()
    }
}

/// https://dom.spec.whatwg.org/#interface-eventtarget
pub trait EventTarget {
    /// https://dom.spec.whatwg.org/#dom-eventtarget-addeventlistener
    fn add_event_listener(&mut self, event_type: String, callback: EventListenerCallback);
    /// https://dom.spec.whatwg.org/#dom-eventtarget-removeeventlistener
    fn remove_event_listener(&mut self, event_type: String, callback: EventListenerCallback);
    /// https://dom.spec.whatwg.org/#dom-eventtarget-dispatchevent
    fn dispatch_event(&mut self, event: Event) -> bool;

    /// Used for compare 2 event targets.
    fn target_kind(&self) -> NodeKind;
}

/// https://dom.spec.whatwg.org/#interface-event
/// https://w3c.github.io/uievents/#uievent
pub enum Event {
    /// https://w3c.github.io/uievents/#idl-mouseevent
    MouseEvent(MouseEvent),
}

/// https://w3c.github.io/uievents/#idl-mouseevent
#[allow(dead_code)]
pub struct MouseEvent {
    event_type: String,
    pub target: Box<dyn EventTarget>,
    screen_x: i32,
    screen_y: i32,
}

impl MouseEvent {
    pub fn new(event_type: String, target: Box<dyn EventTarget>) -> Self {
        Self {
            event_type,
            target,
            screen_x: 0,
            screen_y: 0,
        }
    }

    pub fn event_type(&self) -> String {
        self.event_type.clone()
    }
}
