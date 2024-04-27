//! This is a part of "UI Events" in the W3C specification.
//! DOM Living Standard: https://dom.spec.whatwg.org/#events
//! UI Events W3C Working Draft: https://www.w3.org/TR/uievents/

use alloc::boxed::Box;
use alloc::string::String;

/// https://dom.spec.whatwg.org/#callbackdef-eventlistener
pub type EventListenerCallback = fn(e: Event);

/// https://dom.spec.whatwg.org/#concept-event-listener
#[derive(Debug, Clone)]
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
}

/// https://dom.spec.whatwg.org/#interface-event
#[allow(dead_code)]
pub struct Event {
    event_type: String,
    target: Box<dyn EventTarget>,
}

impl Event {
    /// https://dom.spec.whatwg.org/#concept-event-constructor
    pub fn new(event_type: String, target: Box<dyn EventTarget>) -> Self {
        Self { event_type, target }
    }
}
