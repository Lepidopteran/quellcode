use gtk::{prelude::*, Application, ApplicationWindow, Box, Paned};

mod ui;
mod window;

const APP_ID: &str = "org.quellcode.Quellcode";

pub fn new() -> Application {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);

    app
}

pub fn build_ui(app: &Application) {
    let window = window::Window::new(app);

    window.present();
}
