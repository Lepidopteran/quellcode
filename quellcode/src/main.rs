pub mod app;

use gtk::prelude::*;
use gtk::{gio, glib};

fn main() -> glib::ExitCode {
    dotenvy::dotenv().ok();
    env_logger::init();
    color_eyre::install().ok();
    gtk::init().expect("Failed to initialize GTK");
    gio::resources_register_include!("quellcode.gresource").expect("Failed to register resources");

    let app = app::new();
    app.run()
}
