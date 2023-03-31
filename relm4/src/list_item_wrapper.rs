use std::{
    any::Any,
    cmp::Ordering,
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops,
};

use gtk::{
    gio,
    glib::{self, Bytes},
    prelude::{Cast, CastNone, IsA, ListModelExt, ObjectExt, StaticType},
    SignalListItemFactory,
};

pub trait RelmListItem: Any {
    type Init;
    type Widget: IsA<gtk::Widget>;

    fn init(init: Self::Init) -> Self;
    fn setup() -> Self::Widget;
    fn bind(&self, widget: Self::Widget) {}
    fn unbind(&self, widget: Self::Widget) {}
    fn teardown(list_item: &gtk::ListItem) {}
}

pub struct ListViewWrapper<T> {
    store: gio::ListStore,
    view: gtk::ListView,
    t: PhantomData<*const T>,
}

impl<T: RelmListItem + Ord> ListViewWrapper<T> {
    pub fn with_sorting() -> Self {
        Self::init(Some(Box::new(T::cmp)))
    }
}

impl<T: RelmListItem> ListViewWrapper<T> {
    pub fn new() -> Self {
        Self::init(None)
    }

    fn init(sort_fn: Option<Box<dyn Fn(&T, &T) -> Ordering>>) -> Self {
        let store = gio::ListStore::new(ListItemWrapper::static_type());

        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");

            let widget = T::setup();
            list_item.set_child(Some(&widget));
        });

        factory.connect_bind(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");

            let wrapper = list_item
                .item()
                .and_downcast::<ListItemWrapper>()
                .expect("The item has to be an `IntegerObject`.");
            let obj = wrapper.get::<T>();

            let widget = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem")
                .child();

            obj.bind(widget.and_downcast::<T::Widget>().unwrap());
        });

        factory.connect_unbind(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");

            let wrapper = list_item
                .item()
                .and_downcast::<ListItemWrapper>()
                .expect("The item has to be an `IntegerObject`.");
            let obj = wrapper.get::<T>();

            let widget = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem")
                .child();

            obj.unbind(widget.and_downcast::<T::Widget>().unwrap());
        });

        factory.connect_teardown(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");

            T::teardown(list_item);
        });

        let model: gio::ListModel = store.clone().upcast();

        let model = if let Some(sort_fn) = sort_fn {
            let sorter = gtk::CustomSorter::new(move |first, second| {
                let first = first.downcast_ref::<ListItemWrapper>().unwrap();
                let second = second.downcast_ref::<ListItemWrapper>().unwrap();
                let first = first.get::<T>();
                let second = second.get::<T>();
                match sort_fn(&first, &second) {
                    Ordering::Less => gtk::Ordering::Smaller,
                    Ordering::Equal => gtk::Ordering::Equal,
                    Ordering::Greater => gtk::Ordering::Larger,
                }
            });

            gtk::SortListModel::new(Some(model), Some(sorter)).upcast()
        } else {
            model
        };

        let selection_model = gtk::SingleSelection::new(Some(model));
        let view = gtk::ListView::new(Some(selection_model), Some(factory));

        Self {
            store,
            view,
            t: PhantomData,
        }
    }

    pub fn append(&self, init: T::Init) {
        self.store.append(&ListItemWrapper::new(T::init(init)));
    }

    pub fn view(&self) -> &gtk::ListView {
        &self.view
    }

    pub fn len(&self) -> u32 {
        self.store.n_items()
    }

    pub fn get(&self, position: u32) -> Option<ListItemRef<'_, T>> {
        if let Some(item) = self.store.item(position) {
            let wrapper: &ListItemWrapper = item.downcast_ref().unwrap();
            Some(ListItemRef {
                inner: wrapper.get(),
                _store: &self.store,
            })
        } else {
            None
        }
    }

    pub fn insert(&mut self, position: u32, init: T::Init) {
        self.store
            .insert(position, &ListItemWrapper::new(T::init(init)));
    }

    pub fn insert_sorted<F: FnMut(&T, &T) -> Ordering>(
        &self,
        init: T::Init,
        mut compare_func: F,
    ) -> u32 {
        let item = ListItemWrapper::new(T::init(init));

        let compare = move |first: &glib::Object, second: &glib::Object| -> Ordering {
            let first: &ListItemWrapper = first.downcast_ref().unwrap();
            let second: &ListItemWrapper = second.downcast_ref().unwrap();
            let first = first.get::<T>();
            let second = second.get::<T>();
            compare_func(&first, &second)
        };

        self.store.insert_sorted(&item, compare)
    }

    pub fn remove(&mut self, position: u32) {
        self.store.remove(position);
    }
}

pub struct ListItemRef<'a, T> {
    inner: ManuallyDrop<Box<T>>,
    _store: &'a gio::ListStore,
}

impl<T> ops::Deref for ListItemRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

glib::wrapper! {
    pub struct ListItemWrapper(ObjectSubclass<imp::ListItemWrapper>);
}

impl ListItemWrapper {
    pub fn new<T: Any + 'static>(value: T) -> Self {
        let this: Self = glib::Object::new();
        let (bytes, dropper) = unsafe { AnyWrapper::new(value) };
        this.set_value(bytes);

        this.add_weak_ref_notify_local(move || drop(dropper));

        this
    }

    pub fn get<T: Any>(&self) -> ManuallyDrop<Box<T>> {
        let bytes = self.value().unwrap();
        unsafe { AnyWrapper::from_bytes(bytes) }
    }
}

#[allow(missing_docs)]
mod imp {
    use std::cell::RefCell;

    use glib::prelude::*;
    use glib::{ParamSpec, Properties, Value};
    use gtk::glib::Bytes;
    use gtk::subclass::prelude::ObjectImpl;
    use gtk::{
        glib,
        subclass::prelude::{DerivedObjectProperties, ObjectSubclass},
    };

    #[derive(Default, Properties, Debug)]
    #[properties(wrapper_type = super::ListItemWrapper)]
    /// Inner type of the data binding.
    pub struct ListItemWrapper {
        #[property(get, set)]
        /// The primary value.
        value: RefCell<Option<Bytes>>,
    }

    impl ObjectImpl for ListItemWrapper {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ListItemWrapper {
        const NAME: &'static str = "ListItemWrapper";
        type Type = super::ListItemWrapper;
    }
}

struct AnyWrapper;

impl AnyWrapper {
    unsafe fn new<T: Any + 'static>(inner: T) -> (Bytes, Box<dyn Any>) {
        let value: Box<dyn Any> = Box::new(inner);
        let raw: *mut dyn Any = Box::into_raw(value);

        let bytes: [u8; 16] = unsafe { mem::transmute(raw) };
        let bytes = Bytes::from_owned(bytes);

        let dropper = Box::from_raw(raw);

        (bytes, dropper)
    }

    unsafe fn from_bytes<T: Any>(bytes: Bytes) -> ManuallyDrop<Box<T>> {
        let bytes: Vec<u8> = bytes.iter().copied().collect();
        let bytes: [u8; 16] = bytes.try_into().unwrap();
        let addr: *mut dyn Any = mem::transmute(bytes);
        ManuallyDrop::new(Box::from_raw(addr).downcast().unwrap())
    }
}
