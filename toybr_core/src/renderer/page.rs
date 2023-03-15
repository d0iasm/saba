use crate::browser::Browser;
use crate::common::ui::UiObject;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::css::cssom::*;
use crate::renderer::css::token::*;
use crate::renderer::html::dom::Node;
use crate::renderer::html::dom::*;
use crate::renderer::html::html_builder::dom_to_html;
use crate::renderer::html::token::*;
use crate::renderer::js::ast::JsParser;
use crate::renderer::js::runtime::JsRuntime;
use crate::renderer::js::token::JsLexer;
use crate::renderer::layout::layout_object::LayoutObject;
use crate::renderer::layout::layout_tree_builder::*;
use alloc::rc::{Rc, Weak};
use core::cell::RefCell;
use net::http::HttpResponse;

/// Represents a page. It only supports a main frame.
pub struct Page<U: UiObject> {
    browser: Weak<RefCell<Browser<U>>>,
    url: Option<String>,
    dom_root: Option<Rc<RefCell<Node>>>,
    style: Option<StyleSheet>,
    layout_object_root: Option<Rc<RefCell<LayoutObject>>>,
    modified: bool,
}

impl<U: UiObject> Page<U> {
    pub fn new() -> Self {
        Self {
            browser: Weak::new(),
            url: None,
            dom_root: None,
            style: None,
            layout_object_root: None,
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

        self.set_dom_root(response.body());
        self.set_style();

        self.execute_js();

        while self.modified {
            let dom = match self.dom_root.clone() {
                Some(dom) => dom,
                None => {
                    self.set_layout_object_root();
                    return;
                }
            };

            let modified_html = dom_to_html(&Some(dom));

            self.set_dom_root(modified_html);
            self.set_style();

            self.modified = false;

            self.execute_js();
        }

        self.set_layout_object_root();
    }

    pub fn set_url(&mut self, url: String) {
        self.url = Some(url);
    }

    pub fn set_browser(&mut self, browser: Weak<RefCell<Browser<U>>>) {
        self.browser = browser;
    }

    fn set_dom_root(&mut self, html: String) {
        let html_tokenizer = HtmlTokenizer::new(html);

        let dom_root = HtmlParser::new(self.browser.clone(), html_tokenizer).construct_tree();
        self.dom_root = Some(dom_root);
    }

    fn set_style(&mut self) {
        let dom = match self.dom_root.clone() {
            Some(dom) => dom,
            None => return,
        };

        let style = get_style_content(dom);
        let css_tokenizer = CssTokenizer::new(style);
        let cssom = CssParser::new(self.browser.clone(), css_tokenizer).parse_stylesheet();
        self.style = Some(cssom);
    }

    fn set_layout_object_root(&mut self) {
        let dom = match self.dom_root.clone() {
            Some(dom) => dom,
            None => return,
        };

        let style = match self.style.clone() {
            Some(style) => style,
            None => return,
        };

        let layout_tree = LayoutTree::new(dom, &style);
        self.layout_object_root = layout_tree.root;
    }

    fn execute_js(&mut self) {
        let dom = match self.dom_root.clone() {
            Some(dom) => dom,
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
        self.dom_root.clone()
    }

    pub fn style(&self) -> Option<StyleSheet> {
        self.style.clone()
    }

    pub fn layout_object_root(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.layout_object_root.clone()
    }
}
