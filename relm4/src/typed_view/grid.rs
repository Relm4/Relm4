//! Idiomatic and high-level abstraction over [`gtk::GridView`].

use super::{get_mut_value, get_value, Filter, OrdFn, RelmSelectionExt, TypedListItem};
use gtk::{
    gio, glib,
    prelude::{Cast, CastNone, FilterExt, IsA, ListItemExt, ListModelExt, ObjectExt},
};
use std::{any::Any, cmp::Ordering, marker::PhantomData};

/// An item of a [`TypedGridView`].
pub trait RelmGridItem: Any {
    /// The top-level widget for the grid item.
    type Root: IsA<gtk::Widget>;

    /// The widgets created for the grid item.
    type Widgets;

    /// Construct the widgets.
    fn setup(grid_item: &gtk::ListItem) -> (Self::Root, Self::Widgets);

    /// Bind the widgets to match the data of the grid item.
    fn bind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {}

    /// Undo the steps of [`RelmGridItem::bind()`] if necessary.
    fn unbind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {}

    /// Undo the steps of [`RelmGridItem::setup()`] if necessary.
    fn teardown(_grid_item: &gtk::ListItem) {}
}

/// A high-level wrapper around [`gio::ListStore`],
/// [`gtk::SignalListItemFactory`] and [`gtk::GridView`].
///
/// [`TypedGridView`] aims at keeping nearly the same functionality and
/// flexibility of the raw bindings while introducing a more idiomatic
/// and type-safe interface.
pub struct TypedGridView<T, S> {
    /// The internal grid view.
    pub view: gtk::GridView,
    /// The internal selection model.
    pub selection_model: S,
    store: gio::ListStore,
    filters: Vec<Filter>,
    active_model: gio::ListModel,
    base_model: gio::ListModel,
    _ty: PhantomData<*const T>,
}

impl<T: std::fmt::Debug, S: std::fmt::Debug> std::fmt::Debug for TypedGridView<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedGridView")
            .field("store", &self.store)
            .field("view", &self.view)
            .field("filters", &"<Vec<gtk::Filter>>")
            .field("active_model", &self.active_model)
            .field("base_model", &self.base_model)
            .field("selection_model", &self.selection_model)
            .finish()
    }
}

impl<T, S> TypedGridView<T, S>
where
    T: RelmGridItem + Ord,
    S: RelmSelectionExt,
{
    /// Create a new [`TypedGridView`] that sorts the items
    /// based on the [`Ord`] trait.
    #[must_use]
    pub fn with_sorting() -> Self {
        Self::init(Some(Box::new(T::cmp)))
    }
}

impl<T, S> Default for TypedGridView<T, S>
where
    T: RelmGridItem,
    S: RelmSelectionExt,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, S> TypedGridView<T, S>
where
    T: RelmGridItem,
    S: RelmSelectionExt,
{
    /// Create a new, empty [`TypedGridView`].
    #[must_use]
    pub fn new() -> Self {
        Self::init(None)
    }

    fn init(sort_fn: OrdFn<T>) -> Self {
        let store = gio::ListStore::new::<glib::BoxedAnyObject>();

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");

            let (root, widgets) = T::setup(list_item);
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

            let obj = list_item.item().unwrap();
            let mut obj = get_mut_value::<T>(&obj);

            let mut root = widget.and_downcast::<T::Root>().unwrap();

            let mut widgets = unsafe { root.steal_data("widgets") }.unwrap();
            obj.bind(&mut widgets, &mut root);
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

            let obj = list_item.item().unwrap();
            let mut obj = get_mut_value::<T>(&obj);

            let mut root = widget.and_downcast::<T::Root>().unwrap();

            let mut widgets = unsafe { root.steal_data("widgets") }.unwrap();
            obj.unbind(&mut widgets, &mut root);
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
                let first = get_value::<T>(first);
                let second = get_value::<T>(second);
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

        let selection_model = S::new_model(base_model.clone());
        let view = gtk::GridView::new(Some(selection_model.clone()), Some(factory));

        Self {
            store,
            view,
            filters: Vec::new(),
            active_model: base_model.clone(),
            base_model,
            _ty: PhantomData,
            selection_model,
        }
    }

    /// Add a function to filter the stored items.
    /// Returning `false` will simply hide the item.
    ///
    /// Note that several filters can be added on top of each other.
    pub fn add_filter<F: Fn(&T) -> bool + 'static>(&mut self, f: F) {
        let filter = gtk::CustomFilter::new(move |obj| {
            let value = get_value::<T>(obj);
            f(&value)
        });
        let filter_model =
            gtk::FilterListModel::new(Some(self.active_model.clone()), Some(filter.clone()));
        self.active_model = filter_model.clone().upcast();
        self.selection_model.set_list_model(&self.active_model);
        self.filters.push(Filter {
            filter,
            model: filter_model,
        });
    }

    /// Returns the amount of filters that were added.
    pub fn filters_len(&self) -> usize {
        self.filters.len()
    }

    /// Set a certain filter as active or inactive.
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

    /// Notify that a certain filter has changed.
    /// This causes the filter expression to be re-evaluated.
    pub fn notify_filter_changed(&self, idx: usize) {
        if let Some(filter) = self.filters.get(idx) {
            filter.filter.changed(gtk::FilterChange::Different);
        }
    }

    /// Remove the last filter.
    pub fn pop_filter(&mut self) {
        let filter = self.filters.pop();
        if let Some(filter) = filter {
            self.active_model = filter.model.model().unwrap();
            self.selection_model.set_list_model(&self.active_model);
        }
    }

    /// Remove all filters.
    pub fn clear_filters(&mut self) {
        self.filters.clear();
        self.active_model = self.base_model.clone();
        self.selection_model.set_list_model(&self.active_model);
    }

    /// Add a new item at the end of the list.
    pub fn append(&mut self, value: T) {
        self.store.append(&glib::BoxedAnyObject::new(value));
    }

    /// Add new items from an iterator the the end of the list.
    pub fn extend_from_iter<I: IntoIterator<Item = T>>(&mut self, init: I) {
        let objects: Vec<glib::BoxedAnyObject> =
            init.into_iter().map(glib::BoxedAnyObject::new).collect();
        self.store.extend_from_slice(&objects);
    }

    #[cfg(feature = "gnome_43")]
    #[cfg_attr(docsrs, doc(cfg(feature = "gnome_43")))]
    /// Find the index of the first item that matches a certain function.
    pub fn find<F: FnMut(&T) -> bool>(&self, mut equal_func: F) -> Option<u32> {
        self.store.find_with_equal_func(move |obj| {
            let value = get_value::<T>(obj);
            equal_func(&value)
        })
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the list (without filters).
    pub fn len(&self) -> u32 {
        self.store.n_items()
    }

    /// Get the [`TypedListItem`] at the specified position.
    ///
    /// Returns [`None`] if the position is invalid.
    pub fn get(&self, position: u32) -> Option<TypedListItem<T>> {
        if let Some(obj) = self.store.item(position) {
            let wrapper = obj.downcast::<glib::BoxedAnyObject>().unwrap();
            Some(TypedListItem::new(wrapper))
        } else {
            None
        }
    }

    /// Get the visible [`TypedListItem`] at the specified position,
    /// (the item at the given position after filtering and sorting).
    ///
    /// Returns [`None`] if the position is invalid.
    pub fn get_visible(&self, position: u32) -> Option<TypedListItem<T>> {
        if let Some(obj) = self.active_model.item(position) {
            let wrapper = obj.downcast::<glib::BoxedAnyObject>().unwrap();
            Some(TypedListItem::new(wrapper))
        } else {
            None
        }
    }

    /// Insert an item at a specific position.
    pub fn insert(&mut self, position: u32, value: T) {
        self.store
            .insert(position, &glib::BoxedAnyObject::new(value));
    }

    /// Insert an item into the list and calculate its position from
    /// a sorting function.
    pub fn insert_sorted<F: FnMut(&T, &T) -> Ordering>(
        &self,
        value: T,
        mut compare_func: F,
    ) -> u32 {
        let item = glib::BoxedAnyObject::new(value);

        let compare = move |first: &glib::Object, second: &glib::Object| -> Ordering {
            let first = get_value::<T>(first);
            let second = get_value::<T>(second);
            compare_func(&first, &second)
        };

        self.store.insert_sorted(&item, compare)
    }

    /// Remove an item at a specific position.
    pub fn remove(&mut self, position: u32) {
        self.store.remove(position);
    }

    /// Remove all items.
    pub fn clear(&mut self) {
        self.store.remove_all();
    }
}
