pub mod app;

use gtk::{glib, Application, ApplicationWindow};
use gtk::prelude::*;

fn main() -> glib::ExitCode {
    let app = app::new();
    app.connect_activate(app::build_ui);
    app.run()
}
