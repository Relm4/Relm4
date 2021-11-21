use gtk::prelude::ToVariant;

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

impl<W: gtk::prelude::GtkApplicationExt> AccelsPlus for W {
    fn set_accelerators_for_action<A: ActionName>(&self, accel_codes: &[&str])
    where
        A::Value: EmptyType,
    {
        self.set_accels_for_action(A::action_name().as_str(), accel_codes);
    }
}
