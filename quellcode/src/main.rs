pub mod app;

use gtk::prelude::*;
use gtk::{gio, glib};

fn main() -> glib::ExitCode {
    gio::resources_register_include!("quellcode.gresource").expect("Failed to register resources");

    let app = app::new();

    app.run()
}
