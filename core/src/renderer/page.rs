//! This is corresponding to a page.
//!
//! https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/frame/
//! https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/frame/local_frame_view.h

use crate::alloc::string::ToString;
use crate::browser::Browser;
use crate::http::HttpResponse;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::css::cssom::*;
use crate::renderer::css::token::*;
use crate::renderer::dom::api::{get_js_content, get_style_content};
use crate::renderer::html::dom::Node;
use crate::renderer::html::dom::*;
use crate::renderer::html::html_builder::dom_to_html;
use crate::renderer::html::token::*;
use crate::renderer::js::ast::JsParser;
use crate::renderer::js::runtime::JsRuntime;
use crate::renderer::js::token::JsLexer;
use crate::renderer::layout::layout_view::LayoutView;
use alloc::rc::{Rc, Weak};
use alloc::string::String;
use core::cell::RefCell;

/// Represents a page. It only supports a main frame.
#[derive(Debug, Clone)]
pub struct Page {
    browser: Weak<RefCell<Browser>>,
    url: Option<String>,
    window: Option<Window>,
    style: Option<StyleSheet>,
    layout_view: Option<LayoutView>,
    modified: bool,
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

impl Page {
    pub fn new() -> Self {
        Self {
            browser: Weak::new(),
            url: None,
            window: None,
            style: None,
            layout_view: None,
            modified: false,
        }
    }

    pub fn receive_response(&mut self, response: HttpResponse) {
        let browser = match self.browser.upgrade() {
            Some(browser) => browser,
            None => return,
        };
        browser
            .borrow_mut()
            .console_debug("receive_response start".to_string());

        self.set_window(response.body());
        self.set_style();

        self.execute_js();

        while self.modified {
            let dom = match &self.window {
                Some(window) => window.document(),
                None => {
                    self.set_layout_view();
                    return;
                }
            };

            let modified_html = dom_to_html(&Some(dom));

            self.set_window(modified_html);
            self.set_style();

            self.modified = false;

            self.execute_js();
        }

        self.set_layout_view();

        self.paint_tree();
    }

    pub fn set_url(&mut self, url: String) {
        self.url = Some(url);
    }

    pub fn set_browser(&mut self, browser: Weak<RefCell<Browser>>) {
        self.browser = browser;
    }

    fn set_window(&mut self, html: String) {
        let html_tokenizer = HtmlTokenizer::new(html);

        let window = HtmlParser::new(self.browser.clone(), html_tokenizer).construct_tree();
        self.window = Some(window);
    }

    fn set_style(&mut self) {
        let dom = match &self.window {
            Some(window) => window.document(),
            None => return,
        };

        let style = get_style_content(dom);
        let css_tokenizer = CssTokenizer::new(style);
        let cssom = CssParser::new(self.browser.clone(), css_tokenizer).parse_stylesheet();
        self.style = Some(cssom);
    }

    fn set_layout_view(&mut self) {
        let dom = match &self.window {
            Some(window) => window.document(),
            None => return,
        };

        let style = match self.style.clone() {
            Some(style) => style,
            None => return,
        };

        let layout_view = LayoutView::new(self.browser.clone(), dom, &style);
        self.layout_view = Some(layout_view);
    }

    fn execute_js(&mut self) {
        let dom = match &self.window {
            Some(window) => window.document(),
            None => return,
        };

        let js = get_js_content(dom.clone());
        let lexer = JsLexer::new(js);

        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();

        let url = match self.url.clone() {
            Some(url) => url,
            None => return,
        };

        let mut runtime = JsRuntime::new(dom, url);
        runtime.execute(&ast);

        self.modified = runtime.dom_modified();
    }

    pub fn dom_root(&self) -> Option<Rc<RefCell<Node>>> {
        match &self.window {
            Some(window) => Some(window.document()),
            None => None,
        }
    }

    pub fn style(&self) -> Option<StyleSheet> {
        self.style.clone()
    }

    /*
    pub fn layout_view(&self) -> Option<LayoutView> {
        self.layout_view.clone()
    }
    */

    /// https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/core/frame/local_frame_view.h;drc=0e9a0b6e9bb6ec59521977eec805f5d0bca833e0;bpv=1;bpt=1;l=907
    fn paint_tree(&self) {
        if let Some(layout_view) = &self.layout_view {
            layout_view.paint();
        }
    }
}
