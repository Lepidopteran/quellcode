use gtk::glib;
use gtk::glib::subclass::prelude::*;

use super::Asset;

mod imp {
    use gtk::glib::{subclass::InitializingObject, Properties};
    use gtk::subclass::{
        grid::GridImpl,
        widget::{
            CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
        },
    };
    use gtk::{CompositeTemplate, TemplateChild};

    use super::*;

    #[derive(CompositeTemplate, Properties, Default)]
    #[template(resource = "/org/quellcode/quellcode/asset_details.ui")]
    #[properties(wrapper_type = super::AssetWidget)]
    pub struct AssetWidget {
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub description: TemplateChild<gtk::Label>,
        #[template_child]
        pub installs: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AssetWidget {
        const NAME: &'static str = "QuellcodeStoreAssetDetails";
        type Type = super::AssetWidget;
        type ParentType = gtk::Grid;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AssetWidget {}
    impl WidgetImpl for AssetWidget {}
    impl GridImpl for AssetWidget {}
}

glib::wrapper! {
    pub struct AssetWidget(ObjectSubclass<imp::AssetWidget>)
        @extends gtk::Widget, gtk::Grid,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl AssetWidget {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn bind_data(&self, asset: &Asset) {
        let inner = self.imp();
        inner.name.set_text(&asset.name());
        inner.description.set_text(&asset.description());
        inner.installs.set_text(&asset.installs().to_string());
    }
}

impl Default for AssetWidget {
    fn default() -> Self {
        Self::new()
    }
}
