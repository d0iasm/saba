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
    _kind: String,
    //target: EventTarget,
}

impl MouseEvent {
    pub fn new(_kind: String) -> Self {
        Self { _kind }
    }
}

/// https://www.w3.org/TR/uievents/#events-keyboardevents
/// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/events/keyboard_event.h
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    /// "keydown" and "keyup" are supported.
    /// https://www.w3.org/TR/uievents/#events-keyboard-types
    _kind: String,
    /// https://www.w3.org/TR/uievents/#dom-keyboardevent-key
    _key: String,
    // https://www.w3.org/TR/uievents/#dom-keyboardevent-ctrlkey
    //ctrlKey: bool,
    // https://www.w3.org/TR/uievents/#dom-keyboardevent-shiftkey
    //shiftKey: bool,
}

impl KeyboardEvent {
    pub fn new(_kind: String, _key: String) -> Self {
        Self { _kind, _key }
    }
}
