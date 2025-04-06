use std::any::Any;

use super::{
    column::TypedColumnView,
    grid::{RelmGridItem, TypedGridView},
    list::{RelmListItem, TypedListView},
    selection_ext::RelmSelectionExt,
    TypedListItem,
};

#[derive(Debug)]
/// Holds the state for iterating [`TypedListItem`]s of [`TypedColumnView`], [`TypedGridView`] or [`TypedColumnView`].
pub struct TypedIterator<'a, T> {
    pub(super) list: &'a T,
    pub(super) index: u32,
}

impl<T, S> Iterator for TypedIterator<'_, TypedColumnView<T, S>>
where
    T: Any,
    S: RelmSelectionExt,
{
    type Item = TypedListItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.len() {
            let result = self.list.get(self.index);
            self.index += 1;
            result
        } else {
            None
        }
    }
}

impl<T, S> Iterator for TypedIterator<'_, TypedGridView<T, S>>
where
    T: RelmGridItem + Ord,
    S: RelmSelectionExt,
{
    type Item = TypedListItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.len() {
            let result = self.list.get(self.index);
            self.index += 1;
            result
        } else {
            None
        }
    }
}

impl<T, S> Iterator for TypedIterator<'_, TypedListView<T, S>>
where
    T: RelmListItem,
    S: RelmSelectionExt,
{
    type Item = TypedListItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.len() {
            let result = self.list.get(self.index);
            self.index += 1;
            result
        } else {
            None
        }
    }
}
