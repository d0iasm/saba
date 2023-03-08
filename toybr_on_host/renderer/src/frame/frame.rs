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

/// Represents a page. It only supports a main frame.
pub struct Frame<U: UiObject> {
    ui: Option<Rc<RefCell<U>>>,
    url: String,
    dom_root: Option<Rc<RefCell<Node>>>,
    style: Option<StyleSheet>,
    layout_object_root: Option<Rc<RefCell<LayoutObject>>>,
    modified: bool,
}

impl<U: UiObject> Frame<U> {
    pub fn new(url: String, html: String) -> Self {
        let mut frame = Self {
            ui: None,
            url,
            dom_root: None,
            style: None,
            layout_object_root: None,
            modified: false,
        };

        frame.set_dom_root(html);
        frame.set_style();

        frame.execute_js();

        while frame.modified {
            let dom = match frame.dom_root.clone() {
                Some(dom) => dom,
                None => {
                    frame.set_layout_object_root();
                    return frame;
                }
            };

            let modified_html = dom_to_html(&Some(dom));

            frame.set_dom_root(modified_html);
            frame.set_style();

            frame.modified = false;

            frame.execute_js();
        }

        frame.set_layout_object_root();

        frame
    }

    pub fn set_ui_object(&mut self, ui: Rc<RefCell<U>>) {
        self.ui = Some(ui);
    }

    fn set_dom_root(&mut self, html: String) {
        let html_tokenizer = HtmlTokenizer::new(html);
        let dom_root = HtmlParser::new(html_tokenizer).construct_tree();
        self.dom_root = Some(dom_root);
    }

    fn set_style(&mut self) {
        let dom = match self.dom_root.clone() {
            Some(dom) => dom,
            None => return,
        };

        let style = get_style_content(dom);
        let css_tokenizer = CssTokenizer::new(style);
        let cssom = CssParser::new(css_tokenizer).parse_stylesheet();
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

        let mut runtime = JsRuntime::new(dom, self.url.clone());
        runtime.execute(&ast);

        self.modified = runtime.dom_modified();
    }

    pub fn layout_object_root(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.layout_object_root.clone()
    }
}
