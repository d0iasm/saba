use glib::subclass::InitializingObject;
use glib::subclass::Signal;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, ApplicationWindow, CompositeTemplate, SearchEntry};
use once_cell::sync::Lazy;

#[derive(CompositeTemplate, Default)]
#[template(file = "window.ui")]
pub struct BrowserWindow {
    #[template_child]
    pub entry: TemplateChild<SearchEntry>,
    #[template_child]
    pub content_area: TemplateChild<gtk4::Box>,
}

#[glib::object_subclass]
impl ObjectSubclass for BrowserWindow {
    const NAME: &'static str = "BrowserWindow";
    type Type = super::BrowserWindow;
    type ParentType = ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for BrowserWindow {
    fn constructed(&self, obj: &Self::Type) {
        // Call "constructed" on parent
        self.parent_constructed(obj);

        obj.setup_callbacks();
        obj.setup_actions();
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder(
                // Signal name
                "start-handle-input",
                // Types of the values which will be sent to the signal handler
                &[String::static_type().into()],
                // Type of the value the signal handler sends back
                <()>::static_type().into(),
            )
            .build()]
        });
        SIGNALS.as_ref()
    }
}

impl WidgetImpl for BrowserWindow {}
impl WindowImpl for BrowserWindow {}
impl ApplicationWindowImpl for BrowserWindow {}
