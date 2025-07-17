use gtk::glib;
use gtk::glib::subclass::prelude::*;

use super::Asset;

mod imp {
    use gtk::glib::{subclass::InitializingObject, Properties};
    use gtk::prelude::WidgetExt;
    use gtk::subclass::widget::WidgetImplExt;
    use gtk::subclass::widget::{
        CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
    };
    use gtk::{Allocation, CompositeTemplate, TemplateChild};

    use super::*;

    #[derive(CompositeTemplate, Properties, Default)]
    #[template(resource = "/org/quellcode/quellcode/asset_details.ui")]
    #[properties(wrapper_type = super::AssetWidget)]
    pub struct AssetWidget {
        #[template_child]
        pub container: TemplateChild<gtk::Grid>,
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
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AssetWidget {
        fn dispose(&self) {
            self.container.unparent();
        }
    }

    impl WidgetImpl for AssetWidget {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.container.measure(orientation, for_size)
        }
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.container
                .size_allocate(&Allocation::new(0, 0, width, height), baseline);
        }
    }
}

glib::wrapper! {
    pub struct AssetWidget(ObjectSubclass<imp::AssetWidget>)
        @extends gtk::Widget,
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
        inner.installs.set_text(&{
            let installs = asset.installs();
            if installs >= 1_000_000 {
                format!("{:.1}M", installs as f64 / 1_000_000.0)
            } else if installs >= 1_000 {
                format!("{:.1}K", installs as f64 / 1_000.0)
            } else {
                installs.to_string()
            }
        });
    }
}

impl Default for AssetWidget {
    fn default() -> Self {
        Self::new()
    }
}
