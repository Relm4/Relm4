use gtk::prelude::ToVariant;

/// Trait used to specify the group name in [`ActionName`].
pub trait ActionGroupName {
    /// The name of the group.
    const NAME: &'static str;
}

/// Trait for marking stateless actions.
pub trait EmptyType {}

impl EmptyType for () {}

/// Define the name of an action.
pub trait ActionName {
    /// The group of this action.
    type Group: ActionGroupName;

    /// Target value type for passing values to this action.
    ///
    /// Use [`()`] for actions without target value.
    type Target;

    /// State type of this action.
    ///
    /// Use [`()`] for stateless actions.
    type State;

    /// The name of the action.
    const NAME: &'static str;

    /// The full action name (group.action).
    #[must_use]
    fn action_name() -> String {
        format!("{}.{}", Self::Group::NAME, Self::NAME)
    }
}

/// Type safe interface for [`gtk::prelude::ActionableExt`].
pub trait ActionablePlus {
    /// Set a new stateful action with a default state value.
    fn set_action<A: ActionName>(&self, value: A::Target)
    where
        A::Target: ToVariant;

    /// Set a new stateless action.
    fn set_stateless_action<A: ActionName>(&self, unit_type: &())
    where
        A::Target: EmptyType;
}

impl<W: gtk::prelude::ActionableExt> ActionablePlus for W {
    fn set_action<A: ActionName>(&self, value: A::Target)
    where
        A::Target: ToVariant,
    {
        self.set_action_name(Some(A::action_name().as_str()));
        self.set_action_target_value(Some(&value.to_variant()));
    }

    fn set_stateless_action<A: ActionName>(&self, _unit_type: &())
    where
        A::Target: EmptyType,
    {
        self.set_action_name(Some(A::action_name().as_str()));
    }
}

/// Safe interface for [`gtk::prelude::GtkApplicationExt`].
pub trait AccelsPlus {
    /// Set keyboard accelerator for a certain action.
    fn set_accelerators_for_action<A: ActionName>(&self, value: &[&str])
    where
        A::Target: EmptyType;
}

impl<W: gtk::prelude::GtkApplicationExt> AccelsPlus for W {
    fn set_accelerators_for_action<A: ActionName>(&self, accel_codes: &[&str])
    where
        A::Target: EmptyType,
    {
        self.set_accels_for_action(A::action_name().as_str(), accel_codes);
    }
}
