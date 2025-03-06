use gtk::{glib, Application, ApplicationWindow};
use gtk::prelude::*;

const APP_ID: &str = "org.quellcode.blaine";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Quellcode")
        .build();

    window.present();
}

