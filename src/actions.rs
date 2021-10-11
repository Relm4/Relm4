//! Action utility.

use gtk::prelude::{ActionExt, ActionMapExt, ToVariant, StaticVariantType};
use gtk::{gio, glib};
use glib::variant;

use std::marker::PhantomData;

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
macro_rules! new_statless_action {
    ($ty:ident, $group:ty, $name:expr) => {
        struct $ty;

        impl ActionName for $ty {
            type Group = $group;
            type Value = ();

            fn name() -> &'static str {
                $name
            }
        }
    };
}

#[macro_export]
/// Create a new type that implements [`ActionGroupName`].
macro_rules! new_statful_action {
    ($ty:ident, $group:ty, $name:expr, $value:ty) => {
        struct $ty;

        impl ActionName for $ty {
            type Group = $group;
            type Value = $value;

            fn name() -> &'static str {
                $name
            }
        }
    };
}

/// Trait used to specify the group name in [`ActionName`].
pub trait ActionGroupName {
    /// Returns the group name.
    fn group_name() -> &'static str;
}

/// Trait for marking stateless actions.
pub trait EmptyType {}

impl EmptyType for () {}

/// Define the name of an action.
pub trait ActionName {
    /// The group of this action.
    type Group: ActionGroupName;

    /// Value used for storing the state of this action and
    /// for passing values to it.
    ///
    /// Use [`()`] for stateless actions.
    type Value;

    /// Returns the actions name.
    fn name() -> &'static str;

    /// The full action name (group.action).
    fn action_name() -> String {
        format!("{}.{}", Self::Group::group_name(), Self::name())
    }
}

/// A type safe action that wraps around [`gio::SimpleAction`].
#[derive(Debug)]
pub struct RelmAction<Name: ActionName> {
    name: PhantomData<Name>,
    action: gio::SimpleAction,
}

impl<Name: ActionName> RelmAction<Name>
where
    Name::Value: variant::ToVariant + variant::FromVariant + variant::StaticVariantType + Default,
{
    /// Create a new stateful action.
    pub fn new_stateful<
        Callback: Fn(&gio::SimpleAction, &mut Name::Value, Name::Value) + 'static,
    >(
        start_value: &Name::Value,
        callback: Callback,
    ) -> Self {
        let ty = Name::Value::static_variant_type();

        let action =
            gio::SimpleAction::new_stateful(Name::name(), Some(&ty), &start_value.to_variant());

        action.connect_activate(move |action, variant| {
            let value = variant.unwrap().get::<Name::Value>().unwrap();
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
    Name::Value: EmptyType,
{
    /// Create a new stateless action.
    pub fn new_statelesss<Callback: Fn(&gio::SimpleAction) + 'static>(callback: Callback) -> Self {
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

    /// Create a new [`ActionGroup`].
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

/// Type safe interface for [`gtk::prelude::ActionableExt`].
pub trait ActionablePlus {
    /// Set a new stateful action with a default state value.
    fn set_action<A: ActionName>(&self, value: A::Value)
    where
        A::Value: ToVariant;

    /// Set a new stateless action.
    fn set_stateless_action<A: ActionName>(&self)
    where
        A::Value: EmptyType;
}

impl<W: gtk::prelude::ActionableExt> ActionablePlus for W {
    fn set_action<A: ActionName>(&self, value: A::Value)
    where
        A::Value: ToVariant,
    {
        self.set_action_name(Some(A::action_name().as_str()));
        self.set_action_target_value(Some(&value.to_variant()));
    }

    fn set_stateless_action<A: ActionName>(&self)
    where
        A::Value: EmptyType,
    {
        self.set_action_name(Some(A::action_name().as_str()));
    }
}

/// Safe interface for [`gtk::prelude::GtkApplicationExt`].
pub trait AccelsPlus {
    /// Set keyboard accelerator for a certain action.
    fn set_accelerators_for_action<A: ActionName>(&self, value: &[&str])
    where
        A::Value: EmptyType;
}

impl <W: gtk::prelude::GtkApplicationExt>  AccelsPlus for W {
    fn set_accelerators_for_action<A: ActionName>(&self, accel_codes: &[&str])
    where
        A::Value: EmptyType {
        self.set_accels_for_action(A::action_name().as_str(), accel_codes);
    }
}
