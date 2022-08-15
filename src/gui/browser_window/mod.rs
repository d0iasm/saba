mod imp;

use glib::{clone, Object};
use gtk4::gio::SimpleAction;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gio, glib, Application};

glib::wrapper! {
    pub struct BrowserWindow(ObjectSubclass<imp::BrowserWindow>)
        @extends gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable,
                    gtk4::ConstraintTarget, gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl BrowserWindow {
    pub fn new(app: &Application) -> Self {
        Object::new(&[("application", app)]).expect("Failed to create `BrowserWindow`.")
    }

    fn setup_callbacks(&self) {
        self.imp()
            .entry
            .connect_activate(clone!(@weak self as window => move |entry| {
                window.clear_content_area();
                window.emit_by_name::<()>("start-handle-input", &[&entry.text().to_string()]);
                entry.set_text("");
            }));
    }

    fn setup_actions(&self) {
        let action_count = SimpleAction::new("clicked", Some(&String::static_variant_type()));

        action_count.connect_activate(clone!(@weak self as window => move |_action, parameter| {
            let uri = parameter
                .expect("Could not get parameter.")
                .get::<String>()
                .expect("The value needs to be of type `String`.");

            println!("link clicked and moving to {:?}", uri);

            window.clear_content_area();
            window.emit_by_name::<()>("start-handle-input", &[&uri]);
        }));
        self.add_action(&action_count);
    }

    pub fn get_content_area(&self) -> gtk4::Box {
        self.imp().content_area.get()
    }

    fn clear_content_area(&self) {
        while let Some(child) = self.imp().content_area.get().first_child() {
            self.imp().content_area.get().remove(&child);
        }
    }
}
