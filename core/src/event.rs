//! This is a part of "UI Events" in the W3C specification.
//! https://www.w3.org/TR/uievents/

use alloc::string::String;

#[derive(Debug, Clone)]
pub enum Event {
    /// https://www.w3.org/TR/uievents/#events-mouseevents
    Mouse(MouseEvent),
    /// https://www.w3.org/TR/uievents/#events-keyboardevents
    Keyboard(KeyboardEvent),
}

/// https://www.w3.org/TR/uievents/#events-mouseevents
/// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/events/mouse_event.h
#[derive(Debug, Clone)]
pub struct MouseEvent {
    /// "click", (and TBD) are supported.
    /// https://www.w3.org/TR/uievents/#events-mouse-types
    kind: String,
    //target: EventTarget,
}

impl MouseEvent {
    pub fn new(kind: String) -> Self {
        Self { kind }
    }
}

/// https://www.w3.org/TR/uievents/#events-keyboardevents
/// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/events/keyboard_event.h
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    /// "keydown" and "keyup" are supported.
    /// https://www.w3.org/TR/uievents/#events-keyboard-types
    kind: String,
    /// https://www.w3.org/TR/uievents/#dom-keyboardevent-key
    key: String,
    // https://www.w3.org/TR/uievents/#dom-keyboardevent-ctrlkey
    //ctrlKey: bool,
    // https://www.w3.org/TR/uievents/#dom-keyboardevent-shiftkey
    //shiftKey: bool,
}

impl KeyboardEvent {
    pub fn new(kind: String, key: String) -> Self {
        Self { kind, key }
    }
}
