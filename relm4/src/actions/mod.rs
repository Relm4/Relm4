//! Action utility.

use gtk::gio;
use gtk::prelude::{ActionExt, ActionMapExt, FromVariant, StaticVariantType, ToVariant, WidgetExt};

use std::marker::PhantomData;

/// Type safe traits for interacting with actions.
pub mod traits;
pub use traits::*;

#[macro_export]
/// Create a new type that implements [`ActionGroupName`].
macro_rules! new_action_group {
    ($vis:vis $ty:ident, $name:expr) => {
        #[derive(Clone)]
        $vis struct $ty;

        impl relm4::actions::ActionGroupName for $ty {
            const NAME: &'static str = $name;
        }
    };
}

#[macro_export]
/// Create a new type that implements [`ActionName`] without state or target type.
macro_rules! new_stateless_action {
    ($vis:vis $ty:ident, $group:ty, $name:expr) => {
        #[derive(Clone)]
        $vis struct $ty;

        impl relm4::actions::ActionName for $ty {
            type Group = $group;
            type Target = ();
            type State = ();

            const NAME: &'static str = $name;
        }
    };
}

#[macro_export]
/// Create a new type that implements [`ActionName`] with state and target type.
///
/// The state stores the state of this action and the target type is passed by callers of the action.
macro_rules! new_stateful_action {
    ($vis:vis $ty:ident, $group:ty, $name:expr, $value:ty, $state:ty) => {
        #[derive(Clone)]
        $vis struct $ty;

        impl relm4::actions::ActionName for $ty {
            type Group = $group;
            type Target = $value;
            type State = $state;

            const NAME: &'static str = $name;
        }
    };
}

/// A type safe action that wraps around [`gio::SimpleAction`].
#[derive(Debug, Clone)]
pub struct RelmAction<Name: ActionName> {
    name: PhantomData<Name>,
    action: gio::SimpleAction,
}

impl<Name: ActionName> From<RelmAction<Name>> for gio::SimpleAction {
    fn from(value: RelmAction<Name>) -> Self {
        value.action
    }
}

impl<Name: ActionName> RelmAction<Name>
where
    Name::State: ToVariant + FromVariant,
    Name::Target: ToVariant + FromVariant,
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
            gio::SimpleAction::new_stateful(Name::NAME, Some(&ty), &start_value.to_variant());

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
    Name::State: ToVariant + FromVariant,
    Name::Target: EmptyType,
{
    /// Create a new stateful action.
    pub fn new_stateful<Callback: Fn(&gio::SimpleAction, &mut Name::State) + 'static>(
        start_value: &Name::State,
        callback: Callback,
    ) -> Self {
        let action = gio::SimpleAction::new_stateful(Name::NAME, None, &start_value.to_variant());

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
    Name::Target: ToVariant + FromVariant,
{
    /// Create a new stateless action with a target value.
    pub fn new_with_target_value<Callback: Fn(&gio::SimpleAction, Name::Target) + 'static>(
        callback: Callback,
    ) -> Self {
        let ty = Name::Target::static_variant_type();

        let action = gio::SimpleAction::new(Name::NAME, Some(&ty));

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
        let action = gio::SimpleAction::new(Name::NAME, None);

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
    Name::Target: ToVariant + FromVariant,
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
    #[must_use]
    pub fn to_menu_item(label: &str) -> gio::MenuItem {
        gio::MenuItem::new(Some(label), Some(&Name::action_name()))
    }
}

impl<Name: ActionName> RelmAction<Name> {
    /// Sets the action as enabled or disabled.
    ///
    /// If disabled, the action cannot be activated anymore.
    pub fn set_enabled(&self, enabled: bool) {
        self.action.set_enabled(enabled);
    }

    /// Returns the inner [`gio::SimpleAction`].
    ///
    /// This method is meant for low level control.
    /// Only use it if you know exactly what you are doing.
    #[must_use]
    pub fn gio_action(&self) -> &gio::SimpleAction {
        &self.action
    }
}

#[derive(Debug)]
/// A type-safe action group that wraps around [`gio::SimpleActionGroup`].
pub struct RelmActionGroup<GroupName: ActionGroupName> {
    group_name: PhantomData<GroupName>,
    actions: Vec<gio::SimpleAction>,
}

impl<GroupName: ActionGroupName> RelmActionGroup<GroupName> {
    /// Create a new [`RelmActionGroup`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an action to the group.
    pub fn add_action<Name: ActionName>(&mut self, action: RelmAction<Name>) {
        self.actions.push(action.action);
    }

    /// Register the added actions at application level.
    pub fn register_for_main_application(self) {
        let app = crate::main_application();
        for action in self.actions {
            app.add_action(&action);
        }
    }

    /// Register the added actions for a certain widget.
    pub fn register_for_widget<W>(self, widget: W)
    where
        W: AsRef<gtk::Widget>,
    {
        let group = self.into_action_group();
        widget
            .as_ref()
            .insert_action_group(GroupName::NAME, Some(&group));
    }

    /// Convert [`RelmActionGroup`] into a [`gio::SimpleActionGroup`].
    #[must_use]
    pub fn into_action_group(self) -> gio::SimpleActionGroup {
        let group = gio::SimpleActionGroup::new();
        for action in self.actions {
            group.add_action(&action);
        }
        group
    }
}

impl<GroupName, A> FromIterator<A> for RelmActionGroup<GroupName>
where
    A: Into<gio::SimpleAction>,
    GroupName: ActionGroupName,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = A>,
    {
        Self {
            group_name: PhantomData,
            actions: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl<GroupName: ActionGroupName> Default for RelmActionGroup<GroupName> {
    fn default() -> Self {
        Self {
            group_name: PhantomData,
            actions: Vec::new(),
        }
    }
}
