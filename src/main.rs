mod gui;
mod http;
mod renderer;
mod url;

use crate::http::HttpClient;
use crate::renderer::css::cssom::*;
use crate::renderer::css::token::*;
use crate::renderer::html::dom::*;
use crate::renderer::html::token::*;
use crate::renderer::js::ast::{JsParser, Program};
use crate::renderer::js::runtime::JsRuntime;
use crate::renderer::js::token::JsLexer;
use crate::renderer::layout::render_tree::*;
use crate::url::ParsedUrl;
use core::cell::RefCell;
use std::rc::Rc;
use std::string::String;

/// for debug
fn print_dom(node: &Option<Rc<RefCell<Node>>>, depth: usize) {
    match node {
        Some(n) => {
            print!("{}", "  ".repeat(depth));
            println!("{:?}", n.borrow().kind());
            print_dom(&n.borrow().first_child(), depth + 1);
            print_dom(&n.borrow().next_sibling(), depth);
        }
        None => return,
    }
}

/// for debug
fn print_render_object(node: &Option<Rc<RefCell<RenderObject>>>, depth: usize) {
    match node {
        Some(n) => {
            print!("{}", "  ".repeat(depth));
            println!("{:?} {:?}", n.borrow().kind(), n.borrow().style);
            print_render_object(&n.borrow().first_child(), depth + 1);
            print_render_object(&n.borrow().next_sibling(), depth);
        }
        None => return,
    }
}

fn dom_to_html(node: &Option<Rc<RefCell<Node>>>, html: &mut String) {
    match node {
        Some(n) => {
            // open tag
            match n.borrow().kind() {
                NodeKind::Document => {}
                NodeKind::Element(ref e) => {
                    html.push_str("<");
                    html.push_str(&Element::element_kind_to_string(e.kind()));
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

            // close tag
            match n.borrow().kind() {
                NodeKind::Document => {}
                NodeKind::Element(ref e) => {
                    html.push_str("</");
                    html.push_str(&Element::element_kind_to_string(e.kind()));
                    html.push_str(">");
                }
                NodeKind::Text(_s) => {}
            }

            dom_to_html(&n.borrow().next_sibling(), html);
        }
        None => return,
    }
}

/// for debug
fn print_ast(program: &Program) {
    for node in program.body() {
        println!("{:?}", node);
    }
}

fn handle_input(url: String) -> RenderTree {
    // parse url
    let parsed_url = ParsedUrl::new(url.to_string());
    println!("---------- input url ----------");
    println!("{:?}", parsed_url);

    // send a HTTP request and get a response
    let client = HttpClient::new();
    let response = match client.get(&parsed_url) {
        Ok(res) => {
            println!("status code in HttpResponse: {:?}", res.status_code());

            // redirect to Location
            if res.status_code() == 302 {
                let parsed_redirect_url = ParsedUrl::new(res.header("Location"));

                let redirect_client = HttpClient::new();
                let redirect_res = match redirect_client.get(&parsed_redirect_url) {
                    Ok(res) => res,
                    Err(e) => panic!("failed to get http response: {:?}", e),
                };

                redirect_res
            } else {
                res
            }
        }
        Err(e) => panic!("failed to get http response: {:?}", e),
    };

    println!("---------- http response ----------");
    println!("{:?}", response.body());

    // html
    let html = response.body();
    let html_tokenizer = HtmlTokenizer::new(html);
    let dom_root = HtmlParser::new(html_tokenizer).construct_tree();
    println!("---------- document object model (dom) ----------");
    print_dom(&Some(dom_root.clone()), 0);

    // css
    let style = get_style_content(dom_root.clone());
    //load_css(style.as_bytes());
    let css_tokenizer = CssTokenizer::new(style);
    let cssom = CssParser::new(css_tokenizer).parse_stylesheet();

    println!("---------- css object model (cssom) ----------");
    println!("{:?}", cssom);

    // js
    let js = get_js_content(dom_root.clone());
    let lexer = JsLexer::new(js);

    let mut parser = JsParser::new(lexer);
    let ast = parser.parse_ast();
    println!("---------- javascript abstract syntax tree (ast) ----------");
    print_ast(&ast);

    println!("---------- javascript runtime ----------");
    let mut runtime = JsRuntime::new(dom_root.clone(), url.clone());
    runtime.execute(&ast);

    if runtime.dom_modified() {
        println!("---------- modified document object model (dom) ----------");
        let mut modified_html = String::new();
        dom_to_html(&runtime.dom_root(), &mut modified_html);

        let html_tokenizer = HtmlTokenizer::new(modified_html);
        let modified_dom_root = HtmlParser::new(html_tokenizer).construct_tree();
        print_dom(&Some(modified_dom_root.clone()), 0);

        // apply css to html and create RenderTree
        let render_tree = RenderTree::new(modified_dom_root.clone(), &cssom);
        println!("---------- render tree ----------");
        print_render_object(&render_tree.root, 0);

        return render_tree;
    }

    // apply css to html and create RenderTree
    let render_tree = RenderTree::new(dom_root.clone(), &cssom);
    println!("---------- render tree ----------");
    print_render_object(&render_tree.root, 0);

    return render_tree;
}

fn main() {
    gui::start_browser_window(handle_input);
}
