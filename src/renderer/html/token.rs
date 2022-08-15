//! This is a part of "13.2.5 Tokenization" in the HTML spec.
//! https://html.spec.whatwg.org/multipage/parsing.html#tokenization

use crate::renderer::html::attribute::Attribute;
use core::assert;
use core::iter::Iterator;
use std::string::String;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// https://html.spec.whatwg.org/multipage/parsing.html#data-state
    Data,
    /// https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
    TagOpen,
    /// https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
    EndTagOpen,
    /// https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
    TagName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
    BeforeAttributeName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
    AttributeName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
    AfterAttributeName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
    BeforeAttributeValue,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
    AttributeValueDoubleQuoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
    AttributeValueSingleQuoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
    AttributeValueUnquoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
    AfterAttributeValueQuoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
    SelfClosingStartTag,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
    ScriptData,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state
    ScriptDataLessThanSign,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
    ScriptDataEndTagOpen,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
    ScriptDataEndTagName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#temporary-buffer
    TemporaryBuffer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken {
    // <foo>
    StartTag {
        tag: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
    },
    // </foo>
    EndTag {
        tag: String,
        self_closing: bool,
    },
    // "foo"
    Char(char),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTokenizer {
    state: State,
    pos: usize,
    /// True if the next token should be reconsumed.
    reconsume: bool,
    latest_token: Option<HtmlToken>,
    input: Vec<char>,
    buf: String,
}

impl HtmlTokenizer {
    pub fn new(html: String) -> Self {
        Self {
            state: State::Data,
            pos: 0,
            reconsume: false,
            latest_token: None,
            input: html.chars().collect(),
            buf: String::new(),
        }
    }

    /// Consumes a next input character.
    fn consume_next_input(&mut self) -> char {
        let c = self.input[self.pos];
        self.pos += 1;
        c
    }

    /// Reconsumes the character consumed in a previous step. The `reconsume` flag is reset once
    /// `reconsume_input` is called.
    fn reconsume_input(&mut self) -> char {
        self.reconsume = false;
        self.input[self.pos - 1]
    }

    /// Creates a StartTag or EndTag token.
    fn create_tag_open(&mut self, start_tag_token: bool) {
        if start_tag_token {
            self.latest_token = Some(HtmlToken::StartTag {
                tag: String::new(),
                self_closing: false,
                attributes: Vec::new(),
            });
        } else {
            self.latest_token = Some(HtmlToken::EndTag {
                tag: String::new(),
                self_closing: false,
            });
        }
    }

    /// Appends a char to the tag in the latest created Token `latest_token`.
    fn append_tag_name(&mut self, c: char) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    ref mut tag,
                    self_closing: _,
                    attributes: _,
                }
                | HtmlToken::EndTag {
                    ref mut tag,
                    self_closing: _,
                } => tag.push(c),
                _ => panic!("`latest_token` should be either StartTag or EndTag"),
            }
        }
    }

    /// Starts a new attribute with empty strings in the latest token.
    fn start_new_attribute(&mut self) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    tag: _,
                    self_closing: _,
                    ref mut attributes,
                } => {
                    attributes.push(Attribute::new());
                }
                _ => panic!("`latest_token` should be either StartTag"),
            }
        }
    }

    /// Appends a char to the attribute in the latest created Token `latest_token`.
    fn append_attribute(&mut self, c: char, is_name: bool) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    tag: _,
                    self_closing: _,
                    ref mut attributes,
                } => {
                    let len = attributes.len();
                    assert!(len > 0);

                    attributes[len - 1].add_char(c, is_name);
                }
                _ => panic!("`latest_token` should be either StartTag"),
            }
        }
    }

    /// Sets `self_closing` flag to the `latest_token`.
    fn set_self_closing_flag(&mut self) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag {
                    tag: _,
                    ref mut self_closing,
                    attributes: _,
                }
                | HtmlToken::EndTag {
                    tag: _,
                    ref mut self_closing,
                } => *self_closing = true,
                _ => panic!("`latest_token` should be either StartTag or EndTag"),
            }
        }
    }

    /// Returns `latest_token` and makes it to None.
    fn take_latest_token(&mut self) -> Option<HtmlToken> {
        assert!(self.latest_token.is_some());

        let t = self.latest_token.as_ref().and_then(|t| Some(t.clone()));
        self.latest_token = None;
        assert!(self.latest_token.is_none());

        t
    }

    /// Returns true if the current position is larger than the length of input.
    fn is_eof(&self) -> bool {
        self.pos > self.input.len()
    }

    /// https://html.spec.whatwg.org/multipage/parsing.html#parsing-html-fragments
    pub fn switch_context(&mut self, state: State) {
        self.state = state;
    }
}

impl Iterator for HtmlTokenizer {
    type Item = HtmlToken;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        loop {
            let c = match self.reconsume {
                true => self.reconsume_input(),
                false => self.consume_next_input(),
            };

            match self.state {
                // https://html.spec.whatwg.org/multipage/parsing.html#data-state
                State::Data => {
                    if c == '<' {
                        self.state = State::TagOpen;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    return Some(HtmlToken::Char(c));
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
                State::TagOpen => {
                    if c == '/' {
                        self.state = State::EndTagOpen;
                        continue;
                    }

                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::TagName;
                        self.create_tag_open(true);
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = State::Data;
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
                State::EndTagOpen => {
                    if self.is_eof() {
                        // invalid parse error.
                        return Some(HtmlToken::Eof);
                    }

                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::TagName;
                        self.create_tag_open(false);
                        continue;
                    }
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
                State::TagName => {
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if c.is_ascii_uppercase() {
                        self.append_tag_name(c.to_ascii_lowercase());
                        continue;
                    }

                    if self.is_eof() {
                        // invalid parse error.
                        return Some(HtmlToken::Eof);
                    }

                    self.append_tag_name(c);
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
                State::BeforeAttributeName => {
                    if c == '/' || c == '>' || self.is_eof() {
                        self.reconsume = true;
                        self.state = State::AfterAttributeName;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::AttributeName;
                    self.start_new_attribute();
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
                State::AttributeName => {
                    if c == ' ' || c == '/' || c == '>' || self.is_eof() {
                        self.reconsume = true;
                        self.state = State::AfterAttributeName;
                        continue;
                    }

                    if c == '=' {
                        self.state = State::BeforeAttributeValue;
                        continue;
                    }

                    if c.is_ascii_uppercase() {
                        self.append_attribute(c.to_ascii_lowercase(), /*is_name*/ true);
                        continue;
                    }

                    self.append_attribute(c, /*is_name*/ true);
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
                State::AfterAttributeName => {
                    if c == ' ' {
                        // Ignore.
                        continue;
                    }

                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    if c == '=' {
                        self.state = State::BeforeAttributeValue;
                        continue;
                    }

                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = State::AttributeName;
                    self.start_new_attribute();
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
                State::BeforeAttributeValue => {
                    if c == ' ' {
                        // Ignore the char.
                        continue;
                    }

                    if c == '"' {
                        self.state = State::AttributeValueDoubleQuoted;
                        continue;
                    }

                    if c == '\'' {
                        self.state = State::AttributeValueSingleQuoted;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::AttributeValueUnquoted;
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
                State::AttributeValueDoubleQuoted => {
                    if c == '"' {
                        self.state = State::AfterAttributeValueQuoted;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_attribute(c, /*is_name*/ false);
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
                State::AttributeValueSingleQuoted => {
                    if c == '\'' {
                        self.state = State::AfterAttributeValueQuoted;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_attribute(c, /*is_name*/ false);
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
                State::AttributeValueUnquoted => {
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_attribute(c, /*is_name*/ false);
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
                State::AfterAttributeValueQuoted => {
                    if c == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    if c == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = State::BeforeAttributeValue;
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
                State::SelfClosingStartTag => {
                    if c == '>' {
                        self.set_self_closing_flag();
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        // invalid parse error.
                        return Some(HtmlToken::Eof);
                    }
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
                State::ScriptData => {
                    // TODO: fix this.
                    // this is not aligned with the spec.
                    // check the temporary buffer
                    if c == '>' {
                        if let Some(t) = self.latest_token.as_mut() {
                            match t {
                                HtmlToken::EndTag {
                                    ref tag,
                                    self_closing: _,
                                } => {
                                    if tag == "script" {
                                        self.state = State::Data;
                                        return self.take_latest_token();
                                    } else {
                                        self.latest_token = None;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if c == '<' {
                        self.state = State::ScriptDataLessThanSign;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    return Some(HtmlToken::Char(c));
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state
                State::ScriptDataLessThanSign => {
                    if c == '/' {
                        // "Set the temporary buffer to the empty string."
                        self.buf = String::new();
                        self.state = State::ScriptDataEndTagOpen;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::ScriptData;
                    return Some(HtmlToken::Char('<'));
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
                State::ScriptDataEndTagOpen => {
                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = State::ScriptDataEndTagName;
                        self.create_tag_open(false);
                        continue;
                    }

                    self.reconsume = true;
                    self.state = State::ScriptData;
                    // TODO: emit '<' and '/'
                    // "Emit a U+003C LESS-THAN SIGN character token and a U+002F SOLIDUS character
                    // token. Reconsume in the script data state."
                    return Some(HtmlToken::Char('<'));
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
                State::ScriptDataEndTagName => {
                    if c == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if c.is_ascii_alphabetic() {
                        self.buf.push(c);
                        self.append_tag_name(c.to_ascii_lowercase());
                        continue;
                    }

                    // "Emit a U+003C LESS-THAN SIGN character token, a U+002F SOLIDUS character
                    // token, and a character token for each of the characters in the temporary
                    // buffer (in the order they were added to the buffer). Reconsume in the script
                    // data state."
                    self.state = State::TemporaryBuffer;
                    self.buf = String::from("</") + &self.buf;
                    self.buf.push(c);
                    continue;
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#temporary-buffer
                State::TemporaryBuffer => {
                    self.reconsume = true;

                    if self.buf.chars().count() == 0 {
                        self.state = State::ScriptData;
                        continue;
                    }

                    // remove the first char
                    let c = self
                        .buf
                        .chars()
                        .nth(0)
                        .expect("self.buf should have at least 1 char");
                    self.buf.remove(0);
                    return Some(HtmlToken::Char(c));
                }
            } // end of `match self.state`
        } // end of `loop`
    }
}
