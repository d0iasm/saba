//! This is a part of "UI Events" in the W3C specification.
//! DOM Living Standard: https://dom.spec.whatwg.org/#events
//! UI Events W3C Working Draft: https://www.w3.org/TR/uievents/

use alloc::boxed::Box;
use alloc::string::String;

/// https://dom.spec.whatwg.org/#callbackdef-eventlistener
type EventListener = fn(e: Event);

/// https://dom.spec.whatwg.org/#interface-eventtarget
pub trait EventTarget {
    fn add_event_listener(&self, event_type: String, callback: EventListener);
    fn remove_event_listener(&self, event_type: String, callback: EventListener);
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
