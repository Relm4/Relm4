use gtk::gio;
use gtk::glib::{
    self, Cast, Object, ObjectExt, ParamFlags, ParamSpec, Sender, StaticType, ToValue, Value,
};
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, FilterExt, GtkWindowExt, ListModelExt, SorterExt,
};
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use relm4::*;

use std::cell::Cell;

// Object holding the state
#[derive(Default)]
pub struct GIntegerObject {
    number: Cell<i32>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for GIntegerObject {
    const NAME: &'static str = "MyGtkAppIntegerObject";
    type Type = IntegerObject;
    type ParentType = glib::Object;
}

// Trait shared by all GObjects
impl ObjectImpl for GIntegerObject {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![ParamSpec::new_int(
                // Name
                "number",
                // Nickname
                "number",
                // Short description
                "number",
                // Minimum value
                i32::MIN,
                // Maximum value
                i32::MAX,
                // Default value
                0,
                // The property can be read and written to
                ParamFlags::READWRITE,
            )]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "number" => {
                let input_number = value.get().expect("The value needs to be of type `i32`.");
                self.number.replace(input_number);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "number" => self.number.get().to_value(),
            _ => unimplemented!(),
        }
    }
}

glib::wrapper! {
    pub struct IntegerObject(ObjectSubclass<GIntegerObject>);
}

impl IntegerObject {
    pub fn new(number: i32) -> Self {
        Object::new(&[("number", &number)]).expect("Could not create `IntegerObject`.")
    }

    pub fn increase_number(self) {
        let old_number = self
            .property("number")
            .expect("The property needs to exist and be readable.")
            .get::<i32>()
            .expect("The property needs to be of type `i32`.");

        self.set_property("number", old_number + 1)
            .expect("Could not set property.");
    }
}

struct AppWidgets {
    main: gtk::ApplicationWindow,
}

#[derive(Debug)]
enum AppMsg {
    Add(String),
    RemoveLast,
}

struct AppModel {
    store: gio::ListStore,
}

impl_model!(AppModel, AppMsg);

impl RelmWidgets for AppWidgets {
    type Root = gtk::ApplicationWindow;
    type Model = AppModel;

    fn init_view(model: &AppModel, _components: &(), sender: Sender<AppMsg>) -> Self {
        let main = gtk::ApplicationWindowBuilder::new()
            .default_width(300)
            .default_height(200)
            .build();
        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_end(5)
            .margin_top(5)
            .margin_start(5)
            .margin_bottom(5)
            .spacing(5)
            .build();

        let name = gtk::Entry::builder().placeholder_text("1").build();
        let add = gtk::Button::with_label("Add");
        let remove = gtk::Button::with_label("Remove");

        let scroller = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        main_box.append(&name);
        main_box.append(&add);
        main_box.append(&remove);
        main_box.append(&scroller);

        main.set_child(Some(&main_box));

        let sender2 = sender.clone();
        add.connect_clicked(move |_| {
            let text: String = name.text().into();
            sender2.send(AppMsg::Add(text)).unwrap();
        });

        remove.connect_clicked(move |_| {
            sender.send(AppMsg::RemoveLast).unwrap();
        });

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            // Create label
            let label = gtk::Label::new(None);
            list_item.set_child(Some(&label));

            // Create expression describing `list_item->item->number`
            let list_item_expression = gtk::ConstantExpression::new(list_item);
            let integer_object_expression = gtk::PropertyExpression::new(
                gtk::ListItem::static_type(),
                Some(&list_item_expression),
                "item",
            );
            let number_expression = gtk::PropertyExpression::new(
                IntegerObject::static_type(),
                Some(&integer_object_expression),
                "number",
            );

            // Bind "number" to "label"
            number_expression.bind(&label, "label", Some(&label));
        });

        let filter = gtk::CustomFilter::new(move |obj| {
            // Get `IntegerObject` from `glib::Object`
            let integer_object = obj
                .downcast_ref::<IntegerObject>()
                .expect("The object needs to be of type `IntegerObject`.");

            // Get property "number" from `IntegerObject`
            let _number = integer_object
                .property("number")
                .expect("The property needs to exist and be readable.")
                .get::<i32>()
                .expect("The property needs to be of type `i32`.");

            // Uncomment to only allow even numbers
            // _number % 2 == 0
            true
        });
        let filter_model = gtk::FilterListModel::new(Some(&model.store), Some(&filter));

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            // Get `IntegerObject` from `glib::Object`
            let integer_object_1 = obj1
                .downcast_ref::<IntegerObject>()
                .expect("The object needs to be of type `IntegerObject`.");
            let integer_object_2 = obj2
                .downcast_ref::<IntegerObject>()
                .expect("The object needs to be of type `IntegerObject`.");

            // Get property "number" from `IntegerObject`
            let number_1 = integer_object_1
                .property("number")
                .expect("The property needs to exist and be readable.")
                .get::<i32>()
                .expect("The property needs to be of type `i32`.");
            let number_2 = integer_object_2
                .property("number")
                .expect("The property needs to exist and be readable.")
                .get::<i32>()
                .expect("The property needs to be of type `i32`.");

            // Reverse sorting order -> large numbers come first
            number_2.cmp(&number_1).into()
        });
        let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));

        let selection_model = gtk::SingleSelection::new(Some(&sort_model));
        let list_view = gtk::ListView::new(Some(&selection_model), Some(&factory));

        list_view.connect_activate(move |list_view, position| {
            // Get `IntegerObject` from model
            let model = list_view.model().expect("The model has to exist.");
            let integer_object = model
                .item(position)
                .expect("The item has to exist.")
                .downcast::<IntegerObject>()
                .expect("The item has to be an `IntegerObject`.");

            // Increase "number" of `IntegerObject`
            integer_object.increase_number();

            // Notify that the filter and sorter has been changed
            filter.changed(gtk::FilterChange::Different);
            sorter.changed(gtk::SorterChange::Different);
        });

        scroller.set_child(Some(&list_view));

        AppWidgets { main }
    }

    fn view(&mut self, _model: &AppModel, _sender: Sender<AppMsg>) {}

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
    }
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) {
        match msg {
            AppMsg::Add(text) => {
                let parse_res = text.parse();
                if let Ok(num) = parse_res {
                    self.store.append(&IntegerObject::new(num));
                }
            }
            AppMsg::RemoveLast => {
                let index = self.store.n_items();
                if index != 0 {
                    self.store.remove(index - 1);
                }
            }
        }
    }
}

fn main() {
    let store = gio::ListStore::new(IntegerObject::static_type());
    for number in 0..=10 {
        //0_000 {
        let integer_object = IntegerObject::new(number);
        store.append(&integer_object);
    }

    let model = AppModel { store };
    let relm: RelmApp<AppWidgets> = RelmApp::new(model);
    relm.run();
}
