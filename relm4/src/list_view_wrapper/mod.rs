mod any_wrapper;
mod list_item_wrapper;
mod relm_selection_ext;

use std::{
    any::Any,
    cmp::Ordering,
    marker::PhantomData,
    mem,
    ops,
};

use gtk::{
    gio,
    glib,
    prelude::{Cast, CastNone, IsA, ListModelExt, StaticType, ObjectExt},
    SignalListItemFactory
};

use list_item_wrapper::ListItemWrapper;
use relm_selection_ext::RelmSelectionExt;

use self::list_item_wrapper::get_value_from_wrapper;

pub trait RelmListItem: Any {
    type Init;
    type Root: IsA<gtk::Widget>;
    type Widgets;

    fn init(init: Self::Init) -> Self;
    fn setup() -> (Self::Root, Self::Widgets);
    fn bind(&mut self, root: &Self::Root, widgets: &mut Self::Widgets) {}
    fn unbind(&mut self, root: &Self::Root, widgets: &mut Self::Widgets) {}
    fn teardown(list_item: &gtk::ListItem) {}
}

pub struct ListViewWrapper<T> {
    store: gio::ListStore,
    view: gtk::ListView,
    filters: Vec<Filter>,
    active_model: gio::ListModel,
    base_model: gio::ListModel,
    selection_model: Box<dyn RelmSelectionExt>,
    t: PhantomData<*const T>,
}

impl<T: RelmListItem + Ord> ListViewWrapper<T> {
    pub fn with_sorting() -> Self {
        Self::init(Some(Box::new(T::cmp)))
    }
}

struct Filter {
    filter: gtk::CustomFilter,
    model: gtk::FilterListModel,
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

            let (root, widgets) = T::setup();
            unsafe { root.set_data("widgets", widgets) };
            list_item.set_child(Some(&root));
        });

        factory.connect_bind(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");

            let widget = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem")
                .child();

            let obj = list_item
                .item().unwrap();
            let mut obj = get_value_from_wrapper::<T>(&obj);

            let root = widget.and_downcast::<T::Root>().unwrap();
            let mut widgets: T::Widgets = unsafe { root.steal_data("widgets").unwrap() };
            obj.bind(&root, &mut widgets);
            unsafe { root.set_data("widgets", widgets) };
        });

        factory.connect_unbind(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");

            let widget = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem")
                .child();

            let obj = list_item
                .item().unwrap();
            let mut obj = get_value_from_wrapper::<T>(&obj);


            let root = widget.and_downcast::<T::Root>().unwrap();
            let mut widgets: T::Widgets = unsafe { root.steal_data("widgets").unwrap() };

            obj.unbind(&root, &mut widgets);

            unsafe { root.set_data("widgets", widgets) };
        });

        factory.connect_teardown(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");

            T::teardown(list_item);
        });

        let model: gio::ListModel = store.clone().upcast();

        let base_model = if let Some(sort_fn) = sort_fn {
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

        let selection_model = gtk::SingleSelection::new(Some(base_model.clone()));
        let view = gtk::ListView::new(Some(selection_model.clone()), Some(factory));

        Self {
            store,
            view,
            filters: Vec::new(),
            active_model: base_model.clone(),
            base_model,
            t: PhantomData,
            selection_model: Box::new(selection_model),
        }
    }

    pub fn add_filter<F: Fn(&T) -> bool + 'static>(&mut self, f: F) {
        let filter = gtk::CustomFilter::new(move |obj| {
            let value = get_value_from_wrapper::<T>(obj);
            f(&value)
        });
        let filter_model = gtk::FilterListModel::new(Some(self.active_model.clone()), Some(filter.clone()));
        self.active_model = filter_model.clone().upcast();
        self.selection_model.set_list_model(&self.active_model);
        self.filters.push(Filter {
            filter,
            model: filter_model, 
        });
    }

    pub fn filters_len(&self) -> usize {
        self.filters.len()
    }

    pub fn set_filter_status(&mut self, idx: usize, active: bool) -> bool {
        if let Some(filter) = self.filters.get(idx) {
            if active {
                filter.model.set_filter(Some(&filter.filter));
            } else {
                filter.model.set_filter(None::<&gtk::CustomFilter>);
            }
            true
        } else {
            false
        }
    }

    pub fn pop_filter(&mut self) {
        let filter = self.filters.pop();
        if let Some(filter) = filter {
            self.active_model = filter.model.model().unwrap();
            self.selection_model.set_list_model(&self.active_model);
        }
    }

    pub fn view(&self) -> &gtk::ListView {
        &self.view
    }

    pub fn append(&mut self, init: T::Init) {
        self.store.append(&ListItemWrapper::new(T::init(init)));
    }

    pub fn extend_from_iter<I: IntoIterator<Item = T::Init>>(&mut self, init: I) {
        let objects: Vec<ListItemWrapper> = init.into_iter().map(|init| {
            ListItemWrapper::new(T::init(init))
        }).collect();
        self.store.extend_from_slice(&objects);
    }

    #[cfg(feature = "gnome_43")]
    #[cfg_attr(docsrs, doc(cfg(feature = "gnome_43")))]
    pub fn find<F: FnMut(&T) -> bool>(
        &self,
        mut equal_func: F
    ) -> Option<u32> {
        self.store.find_with_equal_func(move |obj| {
            let value = get_value_from_wrapper::<T>(obj);
            equal_func(&value)
        })
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
            let first = get_value_from_wrapper::<T>(first);
            let second = get_value_from_wrapper::<T>(second);
            compare_func(&first, &second)
        };

        self.store.insert_sorted(&item, compare)
    }

    pub fn remove(&mut self, position: u32) {
        self.store.remove(position);
    }

    pub fn remove_all(&mut self) {
        self.store.remove_all();
    }
}

pub struct ListItemRef<'a, T> {
    inner: mem::ManuallyDrop<Box<T>>,
    _store: &'a gio::ListStore,
}

impl<T> ops::Deref for ListItemRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
