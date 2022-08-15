mod browser_window;

use crate::renderer::html::dom::{ElementKind, NodeKind};
use crate::renderer::layout::render_tree::{DisplayType, FontSize, RenderObject, RenderTree};
use browser_window::BrowserWindow;
use core::cell::RefCell;
use glib::{clone, closure_local};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, Box, DrawingArea, Inhibit, Justification, Label, LinkButton, ListBox,
    Orientation,
};
use std::rc::Rc;

fn should_create_new_box(kind: &NodeKind) -> bool {
    match kind {
        NodeKind::Document => false,
        NodeKind::Element(element) => match element.kind() {
            ElementKind::Html
            | ElementKind::Head
            | ElementKind::Style
            | ElementKind::Script
            | ElementKind::H1
            | ElementKind::H2
            | ElementKind::P => false,
            // TODO: correct?
            ElementKind::Li | ElementKind::Ul => true,
            ElementKind::Body | ElementKind::Div | ElementKind::A => true,
        },
        NodeKind::Text(_) => true,
    }
}

fn paint_render_object(obj: &Rc<RefCell<RenderObject>>, content_area: &Box) {
    match &obj.borrow().kind() {
        NodeKind::Document => {}
        NodeKind::Element(element) => match element.kind() {
            ElementKind::Html
            | ElementKind::Head
            | ElementKind::Style
            | ElementKind::Script
            | ElementKind::H1
            | ElementKind::H2
            | ElementKind::P
            | ElementKind::Body => {}
            ElementKind::Li => {
                let bullet = Label::builder()
                    .label("•")
                    .justify(Justification::Left)
                    .build();
                content_area.append(&bullet);
            }
            ElementKind::Ul => {
                let list_box = ListBox::new();
                content_area.append(&list_box);
            }
            ElementKind::Div => {
                let width = obj.borrow().style.width();
                let height = obj.borrow().style.height();
                let div = DrawingArea::builder()
                    .content_height(height as i32)
                    .content_width(width as i32)
                    .margin_start(obj.borrow().style.margin_left() as i32)
                    .margin_top(obj.borrow().style.margin_top() as i32)
                    .margin_end(obj.borrow().style.margin_right() as i32)
                    .margin_bottom(obj.borrow().style.margin_bottom() as i32)
                    .build();

                let bg_rgb = obj.borrow().style.background_color().rgb();
                let padding_top = obj.borrow().style.padding_top();
                let padding_right = obj.borrow().style.padding_right();
                let padding_bottom = obj.borrow().style.padding_bottom();
                let padding_left = obj.borrow().style.padding_left();
                div.set_draw_func(move |_drawing_area, cairo_context, _w, _h| {
                    cairo_context.rectangle(
                        padding_left as f64,
                        padding_top as f64,
                        (width - padding_right) as f64,
                        (height - padding_bottom) as f64,
                    );
                    cairo_context.set_source_rgb(bg_rgb.0, bg_rgb.1, bg_rgb.2);
                    cairo_context.fill().expect("failed to fill out <div>");
                });
                content_area.append(&div);
            }
            ElementKind::A => {
                let link = LinkButton::builder()
                    .margin_start(obj.borrow().style.margin_left() as i32)
                    .margin_top(obj.borrow().style.margin_top() as i32)
                    .margin_end(obj.borrow().style.margin_right() as i32)
                    .margin_bottom(obj.borrow().style.margin_bottom() as i32)
                    .build();

                let attrs = match obj.borrow().kind() {
                    NodeKind::Element(element) => match element.kind() {
                        ElementKind::A => element.attributes(),
                        _ => Vec::new(),
                    },
                    _ => Vec::new(),
                };

                for attr in attrs {
                    if attr.name == "href" {
                        link.set_uri(&attr.value);
                    }
                }

                link.connect_activate_link(move |link| {
                    let uri: String = link.property("uri");

                    link.activate_action("win.clicked", Some(&uri.to_variant()))
                        .expect("failed to fire win.clicked action");

                    return Inhibit(true);
                });

                content_area.append(&link);
            }
        },
        NodeKind::Text(text) => {
            // Note: this is a hacky way to update label in <a>. This assumes the following
            // structure.
            // |------------------|     |--------------------|
            // | parent_box (Box) | --> | child (LinkButton) |
            // |------------------| |   |--------------------|
            //                      |
            //                      |   |--------------------|
            //                      --> | content_area (Box) |
            //                          |--------------------|
            // Consider smarter implementation
            if let Some(parent_box) = content_area.parent() {
                if let Some(child) = parent_box.first_child() {
                    if child.type_().to_string() == "GtkLinkButton" {
                        child.set_property("label", text);
                        return;
                    }
                }
            }

            let label = Label::builder()
                .justify(Justification::Left)
                .wrap(true)
                .build();

            // https://docs.gtk.org/Pango/pango_markup.html#text-attributes
            let mut markup_attrs = String::new();

            if let Some(color_name) = obj.borrow().style.color().name() {
                markup_attrs.push_str(&format!("foreground=\"{color_name}\" "));
            }

            if obj.borrow().style.font_size() == FontSize::XXLarge {
                markup_attrs.push_str(&format!("size=\"xx-large\""));
            } else if obj.borrow().style.font_size() == FontSize::XLarge {
                markup_attrs.push_str(&format!("size=\"x-large\""));
            }

            // TODO: investigate why this needs.
            label.set_xalign(0.0);

            label.set_markup(&format!("<span {markup_attrs}>{text}</span>"));
            content_area.append(&label);
        }
    }
}

fn paint_render_tree(obj: &Option<Rc<RefCell<RenderObject>>>, parent_content_area: &Box) {
    match obj {
        Some(o) => {
            paint_render_object(o, &parent_content_area);

            if should_create_new_box(&o.borrow().kind()) {
                let new_content_area = if o.borrow().style.display() == DisplayType::Inline {
                    Box::builder()
                        .valign(Align::Start)
                        .halign(Align::Start)
                        .orientation(Orientation::Horizontal)
                        .build()
                } else {
                    Box::builder()
                        .valign(Align::Start)
                        .halign(Align::Start)
                        .width_request(o.borrow().style.width() as i32)
                        .orientation(Orientation::Vertical)
                        .build()
                };

                parent_content_area.append(&new_content_area);

                paint_render_tree(&o.borrow().first_child(), &new_content_area);
                paint_render_tree(&o.borrow().next_sibling(), parent_content_area);
            } else {
                paint_render_tree(&o.borrow().first_child(), parent_content_area);
                paint_render_tree(&o.borrow().next_sibling(), parent_content_area);
            }
        }
        None => return,
    }
}

pub fn start_browser_window(handle_input: fn(String) -> RenderTree) {
    let application = Application::builder().application_id("vulbr").build();

    application.connect_activate(
        clone!(@strong application, @strong handle_input => move |_| {
            let window = BrowserWindow::new(&application);
            window.set_default_size(1280, 800);
            window.set_title(Some("vulbr"));

            window.connect_closure("start-handle-input", false, closure_local!(move |window: BrowserWindow, url: String| {
                println!("start-handle-input {:?}", url);
                let render_tree = handle_input(url);
                paint_render_tree(&render_tree.root, &window.get_content_area());
            }));

            window.show();
        }),
    );

    application.run();
}
