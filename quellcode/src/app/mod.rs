use gtk::{prelude::*, Application, ApplicationWindow, Box, Paned};

pub mod ui;

const APP_ID: &str = "org.quellcode.Quellcode";

pub fn new() -> Application {
    Application::builder()
        .application_id(APP_ID)
        .build()
}

pub fn build_ui(app: &Application) {

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Quellcode")
        .build();

    window.present();
}
