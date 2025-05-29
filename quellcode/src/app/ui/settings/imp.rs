use super::*;
use glib::{subclass::InitializingObject, Properties};
use gtk::{
    glib::subclass::prelude::*,
    subclass::{
        widget::{
            CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
        },
        window::WindowImpl,
    },
    CompositeTemplate, SignalListItemFactory, Stack, TemplateChild,
};

use crate::app::{
    scraping::package_control::{get_packages_by_label, LANGUAGE_SYNTAX},
    tokio_runtime,
};

#[derive(CompositeTemplate, Properties, Default)]
#[template(resource = "/org/quellcode/quellcode/settings.ui")]
#[properties(wrapper_type = super::SettingsWindow)]
pub struct SettingsWindow {
    #[template_child]
    stack: TemplateChild<Stack>,
    #[template_child]
    syntaxes_and_themes_list: TemplateChild<gtk::ListBox>,
}

#[glib::object_subclass]
impl ObjectSubclass for SettingsWindow {
    const NAME: &'static str = "QuellcodeSettingsWindow";
    type Type = super::SettingsWindow;
    type ParentType = gtk::Window;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for SettingsWindow {
    fn constructed(&self) {
        self.parent_constructed();

        let (tx, rx) = async_channel::bounded(1);

        let tx_clone = tx.clone();
        tokio_runtime().spawn(async move {
            let package_list = get_packages_by_label(LANGUAGE_SYNTAX).await.unwrap();
            let _ = tx_clone.send(package_list).await;
        });

        let list_widget = self.syntaxes_and_themes_list.clone();
        glib::spawn_future_local(async move {
            while let Ok(package_list) = rx.recv().await {
                let package_names: Vec<String> = package_list
                    .packages
                    .iter()
                    .map(|p| p.name.clone())
                    .collect();

                for package_name in package_names {
                    let item = gtk::Label::new(Some(&package_name));
                    list_widget.append(&item);
                }
            }
        });
    }
}

impl WidgetImpl for SettingsWindow {}
impl WindowImpl for SettingsWindow {}
