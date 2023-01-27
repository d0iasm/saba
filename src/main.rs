#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

//pub mod http;
pub mod net;
pub mod renderer;
pub mod stdlib;
pub mod url;

use crate::alloc::string::ToString;
use crate::renderer::css::cssom::*;
use crate::renderer::css::token::*;
use crate::renderer::html::dom::*;
use crate::renderer::html::token::*;
use crate::renderer::js::ast::JsParser;
use crate::renderer::js::runtime::JsRuntime;
use crate::renderer::js::token::JsLexer;
use crate::renderer::layout::render_tree::*;
use crate::stdlib::create_window;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use core::panic::PanicInfo;

macro_rules! entry_point {
    // c.f. https://docs.rs/bootloader/0.6.4/bootloader/macro.entry_point.html
    ($path:path) => {
        #[no_mangle]
        pub unsafe extern "C" fn entry() -> i64 {
            // validate the signature of the program entry point
            let f: fn() -> i64 = $path;
            f()
        }
    };
}

entry_point!(main);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unimplemented!();
}

fn dom_to_html(node: &Option<Rc<RefCell<Node>>>, html: &mut String) {
    match node {
        Some(n) => {
            // open a tag
            match n.borrow().kind() {
                NodeKind::Document => {}
                NodeKind::Element(ref e) => {
                    html.push_str("<");
                    html.push_str(&e.kind().to_string());
                    for attr in e.attributes() {
                        html.push_str(" ");
                        html.push_str(&attr.name);
                        html.push_str("=");
                        html.push_str(&attr.value);
                    }
                    html.push_str(">");
                }
                NodeKind::Text(ref s) => html.push_str(s),
            }

            dom_to_html(&n.borrow().first_child(), html);

            // close a tag
            match n.borrow().kind() {
                NodeKind::Document => {}
                NodeKind::Element(ref e) => {
                    html.push_str("</");
                    html.push_str(&e.kind().to_string());
                    html.push_str(">");
                }
                NodeKind::Text(_s) => {}
            }

            dom_to_html(&n.borrow().next_sibling(), html);
        }
        None => return,
    }
}

fn build_render_tree(html: String, url: String) -> Result<RenderTree, String> {
    // html
    let html_tokenizer = HtmlTokenizer::new(html);
    let dom_root = HtmlParser::new(html_tokenizer).construct_tree();

    // css
    let style = get_style_content(dom_root.clone());
    let css_tokenizer = CssTokenizer::new(style);
    let cssom = CssParser::new(css_tokenizer).parse_stylesheet();

    // js
    let js = get_js_content(dom_root.clone());
    let lexer = JsLexer::new(js);

    let mut parser = JsParser::new(lexer);
    let ast = parser.parse_ast();

    let mut runtime = JsRuntime::new(dom_root.clone(), url.clone());
    runtime.execute(&ast);

    if runtime.dom_modified() {
        let mut modified_html = String::new();
        dom_to_html(&runtime.dom_root(), &mut modified_html);

        let html_tokenizer = HtmlTokenizer::new(modified_html);
        let modified_dom_root = HtmlParser::new(html_tokenizer).construct_tree();

        // apply css to html and create RenderTree
        let render_tree = RenderTree::new(modified_dom_root.clone(), &cssom);

        return Ok(render_tree);
    }

    // apply css to html and create RenderTree
    let render_tree = RenderTree::new(dom_root.clone(), &cssom);

    Ok(render_tree)
}

fn main() -> i64 {
    // start a browser window and wait for user activities.
    let _window = create_window();

    let html = default_page();

    let url = "http://example.com";
    let _ = build_render_tree(html, url.to_string());

    return -42;
}

// for debug.
fn default_page() -> String {
    return r#"<html>
<head>
  <style>
    .leaf {
      background-color: green;
      height: 5;
      width: 5;
    }
    #leaf1 {
      margin-top: 50;
      margin-left: 275;
    }
    #leaf2 {
      margin-left: 270;
    }
    #leaf3 {
      margin-left: 265;
    }
    #id2 {
      background-color: orange;
      height: 20;
      width: 30;
      margin-left: 250;
    }
    #id3 {
      background-color: lightgray;
      height: 30;
      width: 80;
      margin-top: 3;
      margin-left: 225;
    }
    #id4 {
      background-color: lightgray;
      height: 30;
      width: 100;
      margin-top: 3;
      margin-left: 215;
    }
  </style>
</head>
<body>
  <div class=leaf id=leaf1></div>
  <div class=leaf id=leaf2></div>
  <div class=leaf id=leaf3></div>
  <div id=id2></div>
  <div id=id3></div>
  <div id=id4></div>
</body>
</html>"#
        .to_string();
}
