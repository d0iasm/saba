use crate::css::cssom::StyleSheet;
use crate::css::cssom::*;
use crate::css::token::*;
use crate::html::dom::Node;
use crate::html::dom::*;
use crate::html::token::*;
use crate::js::ast::JsParser;
use crate::js::runtime::JsRuntime;
use crate::js::token::JsLexer;
use crate::layout::layout_object::LayoutObject;
use crate::layout::layout_tree_builder::*;
use crate::ui::UiObject;
use alloc::rc::Rc;
use core::cell::RefCell;
use net::http::HttpResponse;

/// Represents a page. It only supports a main frame.
pub struct Page<U: UiObject> {
    ui: Option<Rc<RefCell<U>>>,
    url: Option<String>,
    dom_root: Option<Rc<RefCell<Node>>>,
    style: Option<StyleSheet>,
    layout_object_root: Option<Rc<RefCell<LayoutObject>>>,
    modified: bool,
}

impl<U: UiObject> Page<U> {
    pub fn new() -> Self {
        Self {
            ui: None,
            url: None,
            dom_root: None,
            style: None,
            layout_object_root: None,
            modified: false,
        }
    }

    pub fn receive_response(&mut self, response: HttpResponse) {
        let ui = match self.ui.clone() {
            Some(ui) => ui,
            None => return,
        };
        ui.borrow_mut()
            .console_debug("received response".to_string());

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

    pub fn set_ui_object(&mut self, ui: Rc<RefCell<U>>) {
        self.ui = Some(ui);
    }

    fn set_dom_root(&mut self, html: String) {
        let html_tokenizer = HtmlTokenizer::new(html);

        let ui = match self.ui.clone() {
            Some(ui) => ui,
            None => return,
        };

        let dom_root = HtmlParser::new(ui, html_tokenizer).construct_tree();
        self.dom_root = Some(dom_root);
    }

    fn set_style(&mut self) {
        let dom = match self.dom_root.clone() {
            Some(dom) => dom,
            None => return,
        };

        let ui = match self.ui.clone() {
            Some(ui) => ui,
            None => return,
        };

        let style = get_style_content(dom);
        let css_tokenizer = CssTokenizer::new(style);
        let cssom = CssParser::new(ui, css_tokenizer).parse_stylesheet();
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
