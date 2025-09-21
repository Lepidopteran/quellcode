use glib::{subclass::InitializingObject, Properties};
use gtk::{
    glib::subclass::prelude::*,
    subclass::{
        widget::{
            CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
        },
        window::WindowImpl,
    },
    CompositeTemplate, Label, ListItem, SignalListItemFactory, SingleSelection, Stack,
    TemplateChild,
};

use super::*;
use crate::app::{
    asset_store::{Asset, AssetData, AssetWidget},
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
    store_view: TemplateChild<gtk::GridView>,
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

        let model = gio::ListStore::new::<Asset>();

        let model_clone = model.clone();
        glib::spawn_future_local(async move {
            while let Ok(package_list) = rx.recv().await {
                let entries: Vec<Asset> = package_list
                    .packages
                    .into_iter()
                    .map(AssetData::from)
                    .map(Asset::from)
                    .collect();

                model_clone.extend_from_slice(&entries);
            }
        });

        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, item| {
            let details = AssetWidget::new();
            item.downcast_ref::<ListItem>()
                .expect("Item must be a ListItem")
                .set_child(Some(&details));
        });

        factory.connect_bind(move |_, item| {
            let entry = item
                .downcast_ref::<ListItem>()
                .expect("Item must be a ListItem")
                .item()
                .and_downcast::<Asset>()
                .expect("Item must be a StoreEntry");

            let details = item
                .downcast_ref::<ListItem>()
                .expect("Item must be a ListItem")
                .child()
                .and_downcast::<AssetWidget>()
                .expect("Item must be a AssetWidget");

            details.bind_data(&entry);
        });

        let selection_model = SingleSelection::new(Some(model));
        let store_view = self.store_view.clone();

        store_view.set_model(Some(&selection_model));
        store_view.set_factory(Some(&factory));
    }
}

impl WidgetImpl for SettingsWindow {}
impl WindowImpl for SettingsWindow {}
