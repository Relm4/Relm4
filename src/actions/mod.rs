//! Action utility.

use glib::variant;
use gtk::prelude::{ActionExt, ActionMapExt, StaticVariantType, ToVariant};
use gtk::{gio, glib};

use std::marker::PhantomData;

/// Type safe traits for interacting with actions.
pub mod traits;
pub use traits::*;

#[macro_export]
/// Create a new type that implements [`ActionGroupName`].
macro_rules! new_action_group {
    ($ty:ident, $name:expr) => {
        struct $ty;

        impl ActionGroupName for $ty {
            fn group_name() -> &'static str {
                $name
            }
        }
    };
}

#[macro_export]
/// Create a new type that implements [`ActionGroupName`].
macro_rules! new_stateless_action {
    ($ty:ident, $group:ty, $name:expr) => {
        struct $ty;

        impl ActionName for $ty {
            type Group = $group;
            type Target = ();
            type State = ();

            fn name() -> &'static str {
                $name
            }
        }
    };
}

#[macro_export]
/// Create a new type that implements [`ActionGroupName`].
macro_rules! new_stateful_action {
    ($ty:ident, $group:ty, $name:expr, $value:ty, $state:ty) => {
        struct $ty;

        impl ActionName for $ty {
            type Group = $group;
            type Target = $value;
            type State = $state;

            fn name() -> &'static str {
                $name
            }
        }
    };
}

/// A type safe action that wraps around [`gio::SimpleAction`].
#[derive(Debug)]
pub struct RelmAction<Name: ActionName> {
    name: PhantomData<Name>,
    action: gio::SimpleAction,
}

impl<Name: ActionName> RelmAction<Name>
where
    Name::State: variant::ToVariant + variant::FromVariant,
    Name::Target: variant::ToVariant + variant::FromVariant,
{
    /// Create a new stateful action with target value.
    pub fn new_stateful_with_target_value<
        Callback: Fn(&gio::SimpleAction, &mut Name::State, Name::Target) + 'static,
    >(
        start_value: &Name::State,
        callback: Callback,
    ) -> Self {
        let ty = Name::Target::static_variant_type();

        let action =
            gio::SimpleAction::new_stateful(Name::name(), Some(&ty), &start_value.to_variant());

        action.connect_activate(move |action, variant| {
            let value = variant.unwrap().get().unwrap();
            let mut state = action.state().unwrap().get().unwrap();

            callback(action, &mut state, value);
            action.set_state(&state.to_variant());
        });

        Self {
            name: PhantomData,
            action,
        }
    }
}

impl<Name: ActionName> RelmAction<Name>
where
    Name::State: variant::ToVariant + variant::FromVariant,
    Name::Target: EmptyType,
{
    /// Create a new stateful action.
    pub fn new_stateful<Callback: Fn(&gio::SimpleAction, &mut Name::State) + 'static>(
        start_value: &Name::State,
        callback: Callback,
    ) -> Self {
        let action = gio::SimpleAction::new_stateful(Name::name(), None, &start_value.to_variant());

        action.connect_activate(move |action, _variant| {
            let mut state = action.state().unwrap().get().unwrap();
            callback(action, &mut state);
            action.set_state(&state.to_variant());
        });

        Self {
            name: PhantomData,
            action,
        }
    }
}

impl<Name: ActionName> RelmAction<Name>
where
    Name::State: EmptyType,
    Name::Target: variant::ToVariant + variant::FromVariant,
{
    /// Create a new stateless action with a target value.
    pub fn new_with_target_value<Callback: Fn(&gio::SimpleAction, Name::Target) + 'static>(
        callback: Callback,
    ) -> Self {
        let ty = Name::Target::static_variant_type();

        let action = gio::SimpleAction::new(Name::name(), Some(&ty));

        action.connect_activate(move |action, variant| {
            let value = variant.unwrap().get().unwrap();
            callback(action, value);
        });

        Self {
            name: PhantomData,
            action,
        }
    }
}

impl<Name: ActionName> RelmAction<Name>
where
    Name::Target: EmptyType,
    Name::State: EmptyType,
{
    /// Create a new stateless action.
    pub fn new_stateless<Callback: Fn(&gio::SimpleAction) + 'static>(callback: Callback) -> Self {
        let action = gio::SimpleAction::new(Name::name(), None);

        action.connect_activate(move |action, _variant| {
            callback(action);
        });

        Self {
            name: PhantomData,
            action,
        }
    }
}

impl<Name: ActionName> RelmAction<Name>
where
    Name::Target: variant::ToVariant + variant::FromVariant,
{
    /// Create a menu item for this action with the target value sent to the action on activation.
    pub fn to_menu_item_with_target_value(
        label: &str,
        target_value: &Name::Target,
    ) -> gio::MenuItem {
        let menu_item = gio::MenuItem::new(Some(label), Some(&Name::action_name()));
        menu_item.set_action_and_target_value(
            Some(&Name::action_name()),
            Some(&target_value.to_variant()),
        );

        menu_item
    }
}

impl<Name: ActionName> RelmAction<Name>
where
    Name::Target: EmptyType,
{
    /// Create a menu item for this action.
    pub fn to_menu_item(label: &str) -> gio::MenuItem {
        gio::MenuItem::new(Some(label), Some(&Name::action_name()))
    }
}

#[derive(Debug)]
/// A type save action group that wraps around [`gio::SimpleActionGroup`].
pub struct RelmActionGroup<GroupName: ActionGroupName> {
    group_name: PhantomData<GroupName>,
    group: gio::SimpleActionGroup,
}

impl<GroupName: ActionGroupName> RelmActionGroup<GroupName> {
    /// Add an action to the group.
    pub fn add_action<Name: ActionName>(&self, action: RelmAction<Name>) {
        self.group.add_action(&action.action);
    }

    /// Convert [`RelmActionGroup`] into a [`gio::SimpleActionGroup`].
    pub fn into_action_group(self) -> gio::SimpleActionGroup {
        self.group
    }

    /// Create a new [`SimpleActionGroup`](gio::SimpleActionGroup).
    pub fn new() -> Self {
        Self {
            group_name: PhantomData,
            group: gio::SimpleActionGroup::new(),
        }
    }
}

impl<GroupName: ActionGroupName> Default for RelmActionGroup<GroupName> {
    fn default() -> Self {
        Self::new()
    }
}
