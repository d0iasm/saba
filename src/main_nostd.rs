#![no_std]
#![no_main]
#![feature(start)]
#![feature(alloc_error_handler)]

extern crate alloc;

//pub mod http;
pub mod net;
//pub mod renderer;
pub mod stdlib;
//pub mod url;

use crate::alloc::string::ToString;
//use crate::renderer::layout::render_tree::RenderTree;
use crate::stdlib::create_window;
use alloc::string::String;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unimplemented!();
}

#[no_mangle]
fn entry() -> isize {
    start_browser();

    -42
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

fn start_browser() {
    // Create a browser window that watches user actions (mouse move, mouse click, etc.).
    let _window = create_window();

    // Send a request to the initial page.
    println!("aaa");
    let html = default_page();

    /*
    let response = match get_http_response(url.clone()) {
        Ok(res) => res,
        Err(error_message) => return build_error_render_tree(error_message, url.clone()),
    };

    build_render_tree(response.body(), url)
    */
}

/*
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
*/
