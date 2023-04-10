/*
 * SPDX-FileCopyrightText: 2023 Eduardo Javier Alvarado Aarón <eduardo.javier.alvarado.aaron@gmail.com>
 *
 * SPDX-License-Identifier: (Apache-2.0 or MIT)
 */

//! Utilities to safely manage actions and settings.

#![allow(unused_qualifications)]

use gio::prelude::{ActionExt, IsA, SettingsExt, SettingsExtManual, ToVariant};
use gtk::{gio, glib};

/// The safe extensions traits, without the safety traits.
pub mod prelude {
    pub use super::{
        RelmMenu, SafeAction, SafeActionMap, SafeActionable, SafeApplication, SafeSettings,
        SafeSimpleAction,
    };

    #[cfg(feature = "libadwaita")]
    pub use super::SafeToast;
}

/// The basic trait for declaring setting safeties.
pub trait SettingSafety {
    /// The name of the setting or action.
    ///
    /// The macro [`safeties!`] automatically implements [`Display`](std::fmt::Display) using
    /// this value, so it is not necessary to use it directly for loggin and similar cases.
    const NAME: &'static str;

    // NOTE: This method should be inside ActionSafety, but macro_rules! won't let me.
    /// The `Variant` type concerning the [`Value`](WithValue::Value) of this safety.
    fn variant_type() -> Option<std::borrow::Cow<'static, glib::VariantTy>>;
}

/// The basic trait for declaring action safeties.
pub trait ActionSafety: SettingSafety {
    /// The action name plus the group, as follows: `group.name`
    const FULL_NAME: &'static str;

    /// A self value.
    const SELF: Self;
}

/// A trait that extends [`SettingSafety`] for safeties without value.
///
/// Any type that implements it must be [`DetailableSetting`] and must not implement [`WithValue`].
pub trait WithoutValue: DetailableSetting {}

/// A trait that extends [`SettingSafety`] for safeties with value.
///
/// Any type that implements it must not implement [`WithoutValue`].
pub trait WithValue<'a>: SettingSafety {
    /// Type used as parameter in safe methods.
    type Value: glib::ToVariant + Into<glib::Variant>;

    /// Type used to receive the value on signals.
    type Mapping;

    /// Function to convert [`glib::Variant`] to the [`Mapping`](WithValue::Mapping) type.
    fn map(variant: &'a glib::Variant) -> Self::Mapping;
}

/// A trait that extends [`SettingSafety`] for safeties without state.
///
/// Any type that implements it must not implement [`Stateful`].
pub trait Stateless: SettingSafety {}

/// A trait that extends [`SettingSafety`] for safeties with state.
///
/// Any type that implements it must not implement [`Stateless`].
pub trait Stateful<'a>: SettingSafety {
    /// Type used as parameter in safe methods.
    type State: glib::ToVariant;

    /// Owned version of the [`State`](Stateful::State) type, used by getters or methods that mutate state.
    type Owned: glib::ToVariant + glib::FromVariant;

    /// Type used to receive the state on signals.
    type Mapping;

    /// Function to convert [`glib::Variant`] to the [`Mapping`](Stateful::Mapping) type.
    fn map(variant: &'a glib::Variant) -> Self::Mapping;
}

/// Trait for setting or action safeties with value and without variants.
///
/// Any type that implements it must not implement [`DetailableSetting`].
pub trait NotDetailable: SettingSafety {}

/// Trait for setting safeties with variants, or without value and variants.
///
/// Detailable setting safeties facilitate settings maintenance.
///
/// Any type that implements it must not implement [`NotDetailable`].
///
/// ### Example:
/// ~~~
/// use relm4::gtk::prelude::{StaticVariantType, ToVariant};
///
/// relm4::safeties! {
///     // Implements DetailableSetting:
///     @state(param: bool)
///     MySetting(name: "my-setting");
///
///     // Implements DetailableSetting:
///     @value(param: &'a str, map: <str>)
///     ColorScheme(name: "color-scheme") {
///         System = ("System"),
///         Light  = ("Light"),
///         Dark   = ("Dark"),
///     }
///
///     // Does not implement DetailableSetting:
///     @value(param: i32)
///     NotDetailable(name: "not-detailable");
/// }
/// ~~~
/// See also: [`safeties!`], [`NotDetailable`].
pub trait DetailableSetting: SettingSafety {
    /// Function to convert [`glib::Variant`] to `Self`, especially when `Self` is an `enum`.
    fn from_variant(variant: &glib::Variant) -> Self
    where
        Self: for<'a> WithValue<'a>;

    /// Function to convert `Self` to [`glib::Variant`], especially when `Self` is an `enum`.
    fn to_variant(&self) -> Option<glib::Variant>;
}

/// Trait for action safeties with variants, or without value and variants.
///
/// Detailable action safeties facilitate keyboard accelerators and settings maintenance.
///
/// Any type that implements it must not implement [`NotDetailable`].
///
/// ### Example:
/// ~~~
/// use relm4::gtk::prelude::{StaticVariantType, ToVariant};
///
/// relm4::safeties! {
///     // Implements DetailableAction:
///     MyAction(group: "win", name: "my-action");
///
///     // Implements DetailableAction:
///     @state(param: bool)
///     MySetting(group: "win", name: "my-setting");
///
///     // Implements DetailableAction:
///     @value(param: &'a str, map: <str>)
///     ColorScheme(group: "win", name: "color-scheme") {
///         System = ("System"),
///         Light  = ("Light"),
///         Dark   = ("Dark"),
///     }
///
///     // Does not implement DetailableAction:
///     @value(param: i32)
///     NotDetailable(group: "win", name: "not-detailable");
/// }
/// ~~~
/// See also: [`safeties!`], [`NotDetailable`].
pub trait DetailableAction: ActionSafety + DetailableSetting {
    /// Gets the [detailed action name](https://docs.gtk.org/gio/type_func.Action.parse_detailed_name.html).
    fn detailed_action_name(&self) -> &'static str;
}

#[macro_export]
/// A macro to create unit structs or enums that implement [`SettingSafety`] / [`ActionSafety`],
/// and depending on their characteristics also implements [`WithValue`] or [`WithoutValue`],
/// [`Stateful`] or [`Stateless`], and [`DetailableSetting`] / [`DetailableAction`] or [`NotDetailable`].
///
/// Safeties can have a `@value`, which refers to the target value of
/// some actionable methods or to settings values, as well as a `@state`.
///
/// `@value` and `@state` have the following parameters:
/// 1. `param`: specifies the main data type and is used as a parameter of some safe methods. It is mandatory.
/// 2. `owned`: an owned version of the param type. If not specified, the same as param is used.
/// 3. `map`: a method of [`glib::Variant`] to convert it to the correct type data, used to receive the
///           `@value` or `@state` in signals, or to retrieve an enumerated variant from [`glib::Variant`].
///
/// Unlike `param` and `owned`, `map` is specified as follows: `<method> ReturnType`.
/// If not specified, it falls back to: `<get> OwnedType`.
/// If only the method and not the return type is specified, falls back to: `<method> ParamType`.
///
/// In the case of `@value` it is not necessary `owned` if `map` is specified.
///
/// ### Syntax
/// ~~~
/// use relm4::gtk::prelude::{StaticVariantType, ToVariant};
///
/// relm4::safeties! {
///     // Action safeties must specify the group parameter.
///     UnitStruct(group: "action-group", name: "action-or-setting-name");
///
///     // If a lifetime is necessary, use 'a.
///     @value(param: &'a str, owned: String, map: <str> &'a str)
///     @state(param: &'a str, owned: String, map: <str> &'a str)
///     Enum(name: "setting-name") {
///         Variant = ("literal-value"), // ... Variants require @value.
///         // In the case of string literals, they must always be enclosed
///         // in parens for keyboard accelerators and settings to work.
///     }
/// }
/// ~~~
macro_rules! safeties {
    (fallback! @($($foo:tt)+)            ) => { $($foo)+ };
    (fallback! @($($foo:tt)+) $($bar:tt)+) => { $($bar)+ };

    ( value:     $value:literal      ) => { $value };
    ( value: (   $value:literal     )) => { $value };
    ( value: [ $($value:tt),+ $(,)? ]) => { &[$($crate::safeties!(value: $value)),+] };
    ( value: ( $($value:tt),+ $(,)? )) => {  ($($crate::safeties!(value: $value)),+) };
    (detail:     $value:literal      ) => { $value };
    (detail: (   $value:literal     )) => { concat!("'", $value, "'") };
    (detail: [ $($value:tt),+ $(,)? ]) => { concat!("[" $(,$crate::safeties!(detail: $value),)','+ "]") };
    (detail: ( $($value:tt),+ $(,)? )) => { concat!("(" $(,$crate::safeties!(detail: $value),)','+ ")") };

    (setting: $name:literal;) => {
        fn from_variant(_: &$crate::gtk::glib::Variant) -> Self { Self }
        fn to_variant(&self) -> Option<$crate::gtk::glib::Variant> { None }
    };
    (action: $group:literal $name:literal;) => {
        fn detailed_action_name(&self) -> &'static str {
            <Self as $crate::safeties::ActionSafety>::FULL_NAME
        }
    };
    (setting: $name:literal { $($variant:ident = $value:tt),+ $(,)? }) => {
        fn from_variant(variant: &$crate::gtk::glib::Variant) -> Self {
            match <Self as $crate::safeties::WithValue>::map(&variant) {
                $($crate::safeties!(value: $value) => Self::$variant,)+
                _ => unreachable!()
            }
        }
        fn to_variant(&self) -> Option<$crate::gtk::glib::Variant> {
            Some(match self {
                $(Self::$variant => $crate::safeties!(value: $value).to_variant()),+
            })
        }
    };
    (action: $group:literal $name:literal { $($variant:ident = $value:tt),+ $(,)? }) => {
        fn detailed_action_name(&self) -> &'static str {
            match self {
                $(Self::$variant => concat!($group, '.', $name, '(', $crate::safeties!(detail: $value), ')')),+
            }
        }
    };

    (safety!
        $(#[$attr:meta])*
        $(@value(
            param:  $target_param:ty $(,
            owned:  $target_owned:ty)? $(,
              map: <$target_map:ident> $($target_mapping:ty)?)? $(,)?
        ))?
        $(@state(
            param:  $state_param:ty $(,
            owned:  $state_owned:ty)? $(,
              map: <$state_map:ident> $state_mapping:ty)? $(,)?
        ))?
        $vis:vis $type:ident($(group: $group:literal,)? name: $name:literal $(,)?) $variants:tt
    ) => {
        $crate::safeties!(self_type! $(#[$attr])* $vis $type $variants);

        impl $crate::safeties::SettingSafety for $type {
            const NAME: &'static str = $name;
            fn variant_type<'a>() -> Option<std::borrow::Cow<'static, $crate::gtk::glib::VariantTy>> {
                $crate::safeties!(fallback! @(None) $(Some(<$target_param>::static_variant_type()))?)
            }
        }

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", <$type as $crate::safeties::SettingSafety>::NAME)
            }
        }

        $(impl $crate::safeties::ActionSafety for $type {
            const FULL_NAME: &'static str = concat!($group, '.', $name);
            const SELF: Self = $crate::safeties!(self_constant! $type $variants);
        })?

        $crate::safeties! { fallback!
            @(impl $crate::safeties::WithoutValue for $type { })
            $(impl<'a> $crate::safeties::WithValue<'a> for $type {
                type Value = $target_param;
                type Mapping = $crate::safeties!(fallback! @(
                    $crate::safeties!(fallback! @($target_param) $($target_owned)?)
                ) $($($target_mapping)?)?);

                $crate::safeties! { fallback!
                    @(fn map(variant: &'a $crate::gtk::glib::Variant) -> $crate::safeties! {
                        fallback! @($target_param) $($target_owned)?
                    } { variant.get().unwrap() })

                    $(fn map(variant: &'a $crate::gtk::glib::Variant) -> Self::Mapping {
                        variant.$target_map().unwrap()
                    })?
                }
            })?
        }

        $crate::safeties! { fallback!
            @(impl $crate::safeties::Stateless for $type { })
            $(impl<'a> $crate::safeties::Stateful<'a> for $type {
                type State   = $state_param;
                type Owned   = $crate::safeties!(fallback! @($state_param) $($state_owned)?);
                type Mapping = $crate::safeties!(fallback! @(Self::Owned) $($state_mapping)?);

                $crate::safeties! { fallback!
                    @(fn map(variant: &'a $crate::gtk::glib::Variant) -> $crate::safeties! {
                        fallback! @($state_param) $($state_owned)?
                    } { variant.get().unwrap() })

                    $(fn map(variant: &'a $crate::gtk::glib::Variant) -> Self::Mapping {
                        variant.$state_map().unwrap()
                    })?
                }
            })?
        }

        $crate::safeties! { detailable_or_not!
            $type [$($target_param)?] [$variants]
            impl $crate::safeties::DetailableSetting for $type {
                $crate::safeties!(setting: $name $variants);
            }
            $(impl $crate::safeties::DetailableAction for $type {
                $crate::safeties!(action: $group $name $variants);
            })?
            #[allow(dead_code)]
            impl $type {
                $crate::safeties!(value! $(
                    $crate::safeties!(fallback! @($target_param) $($($target_mapping)?)?)
                )? $variants);
            }
        }
    };

    (self_type! $(#[$attr:meta])* $vis:vis $type:ident;) => { $(#[$attr])* $vis struct $type; };
    (self_type! $(#[$attr:meta])* $vis:vis $type:ident { $($variant:ident = $value:tt),* $(,)? }) => {
        $(#[$attr])* $vis enum $type { $($variant),* }
    };

    (self_constant! $type:ident;) => { $type };
    (self_constant! $type:ident { $variant:ident = $value:tt $(,$variants:ident = $values:tt)* $(,)? }) => { $type::$variant };

    (detailable_or_not! $type:ty [$target:ty] [         ;  ] $($impl:tt)+) => { impl $crate::safeties::NotDetailable for $type { } };
    (detailable_or_not! $type:ty [          ] [         ;  ] $($impl:tt)+) => { $($impl)+ };
    (detailable_or_not! $type:ty [$target:ty] [$variants:tt] $($impl:tt)+) => { $($impl)+ };

    (value! ;) => { };
    (value! $target:ty { $($variant:ident = $value:tt),+ $(,)? }) => {
        const fn value<'a>(&self) -> $target {
            match self { $(Self::$variant => $crate::safeties!(value: $value)),+ }
        }
        const fn some<'a>(&self) -> Option<$target> {
            Some(match self { $(Self::$variant => $crate::safeties!(value: $value)),+ })
        }
    };

    ($($(#[$attr:meta])* $(@$keyword:tt $params:tt)* $vis:vis $type:ident $naming:tt $variants:tt)*) => {
        $($crate::safeties!(safety! $(#[$attr])* $(@$keyword $params)* $vis $type $naming $variants);)*
    };
}

/// Trait that extends [`gio::Action`] with action safety methods.
pub trait SafeAction: gio::prelude::ActionExt {
    /// Safe version of [`state`](gio::prelude::ActionExt::state) for stateful action safeties.
    fn state_safe<'a, T: ActionSafety + Stateful<'a>>(&self, _safety: T) -> T::Owned {
        self.state().unwrap().get().unwrap()
    }

    /// Safe version of [`state`](gio::prelude::ActionExt::state) for action safeties with variants.
    fn state_safe_enum<T: for<'a> WithValue<'a> + DetailableAction>(&self) -> T {
        T::from_variant(&self.state().unwrap())
    }

    /// Safe version of [`connect_state_notify`](gio::prelude::ActionExt::connect_state_notify) for stateful action safeties.
    fn connect_state_notify_safe<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> Stateful<'a>,
        F: Fn(T, &Self, <T as Stateful<'_>>::Mapping) + 'static,
    {
        self.connect_state_notify(move |this| {
            callback(T::SELF, this, T::map(&this.state().unwrap()))
        })
    }

    /// Safe version of [`connect_state_notify`](gio::prelude::ActionExt::connect_state_notify) for action safeties with variants.
    fn connect_state_notify_safe_enum<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: for<'a> WithValue<'a> + DetailableAction,
        F: Fn(&Self, T) + 'static,
    {
        self.connect_state_notify(move |this| {
            callback(this, T::from_variant(&this.state().unwrap()))
        })
    }
}

impl<T: IsA<gio::Action>> SafeAction for T {}

/// Trait that extends [`gtk::Actionable`] with action safety methods.
pub trait SafeActionable: gtk::prelude::ActionableExt {
    /// Appropriately assigns the [name][n] and [target value][t] of an action
    /// according to an action safety with variants or without value nor variants.
    ///
    /// [n]: gtk::prelude::ActionableExt::set_action_name
    /// [t]: gtk::prelude::ActionableExt::set_action_target_value
    fn set_action_safe<T: DetailableAction>(&self, safety: T) {
        // self.set_detailed_action_name(safety.detailed_action_name());
        self.set_action_name(Some(T::FULL_NAME));
        self.set_action_target_value(safety.to_variant().as_ref());
    }

    /// Appropriately assigns the [name][n] and [target value][t] of an
    /// action according to an action safety with value and without variants.
    ///
    /// [n]: gtk::prelude::ActionableExt::set_action_name
    /// [t]: gtk::prelude::ActionableExt::set_action_target_value
    fn set_target_safe<'a, T: WithValue<'a>>(&self, _safety: T, target: T::Value)
    where
        T: ActionSafety + NotDetailable,
    {
        self.set_action_name(Some(T::FULL_NAME));
        self.set_action_target_value(Some(&target.to_variant()))
    }
}

impl<T: IsA<gtk::Actionable>> SafeActionable for T {}

/// Trait that extends [`gio::ActionMap`] with action safety methods.
pub trait SafeActionMap: gio::prelude::ActionMapExt {
    /// Safe version of [`lookup_action`](gio::prelude::ActionMapExt::lookup_action) for action safeties.
    fn lookup_action_safe<T: ActionSafety>(&self, _safety: T) -> Option<gio::Action> {
        self.lookup_action(T::NAME)
    }

    /// Safe version of [`remove_action`](gio::prelude::ActionMapExt::remove_action) for action safeties.
    fn remove_action_safe<T: ActionSafety>(&self, _safety: T) {
        self.remove_action(T::NAME)
    }
}

impl<T: IsA<gio::ActionMap>> SafeActionMap for T {}

/// Trait that extends [`gtk::Application`] with action safety methods.
pub trait SafeApplication: gtk::prelude::GtkApplicationExt {
    /// Safe version of [`accels_for_action`](gtk::prelude::GtkApplicationExt::accels_for_action)
    /// for action safeties with variants or without value nor variants.
    fn accels_for_action_safe<T: DetailableAction>(&self, safety: T) -> Vec<glib::GString> {
        self.accels_for_action(safety.detailed_action_name())
    }

    /// Safe version of [`set_accels_for_action`](gtk::prelude::GtkApplicationExt::set_accels_for_action)
    /// for action safeties with variants or without value nor variants.
    fn set_accels_for_action_safe<T: DetailableAction>(&self, safety: T, accels: &[&str]) {
        self.set_accels_for_action(safety.detailed_action_name(), accels)
    }
}

impl<T: IsA<gtk::Application>> SafeApplication for T {}

#[cfg(feature = "macros")]
/// Trait that extends [`gio::Menu`] with methods compatible with [`relm4_macros::view!`] and action safety methods.
pub trait RelmMenu: IsA<gio::Menu> {
    /// Adds a menu item for an action safety with variants or without value nor variants.
    fn action<T: DetailableAction>(&self, safety: T, label: &str);

    /// Adds a menu item for an action safety with value and without variants.
    fn target<'a, T: WithValue<'a>>(&self, safety: T, target: T::Value, label: &str)
    where
        T: ActionSafety + NotDetailable;

    /// Adds a menu item for a detailed action name.
    fn detailed(&self, action: &str, label: &str);

    /// Adds a section.
    fn section(&self, model: &impl IsA<gio::MenuModel>, label: &str);

    /// Adds a submenu.
    fn submenu(&self, model: &impl IsA<gio::MenuModel>, label: &str);

    /// Adds a placeholder for a widget.
    fn widget(&self, label: &str);
}

impl RelmMenu for gio::Menu {
    fn action<T: DetailableAction>(&self, safety: T, label: &str) {
        // self.append(Some(label), Some(safety.detailed_action_name()));
        let item = gio::MenuItem::new(Some(label), None);
        item.set_action_and_target_value(Some(T::FULL_NAME), safety.to_variant().as_ref());
        self.append_item(&item);
    }

    fn target<'a, T: WithValue<'a>>(&self, _safety: T, target: T::Value, label: &str)
    where
        T: ActionSafety,
    {
        let item = gio::MenuItem::new(Some(label), None);
        item.set_action_and_target_value(Some(T::FULL_NAME), Some(&target.to_variant()));
        self.append_item(&item);
    }

    fn detailed(&self, action: &str, label: &str) {
        self.append(Some(label), Some(action));
    }

    fn section(&self, model: &impl IsA<gio::MenuModel>, label: &str) {
        self.append_section((!label.is_empty()).then_some(label), model);
    }

    fn submenu(&self, model: &impl IsA<gio::MenuModel>, label: &str) {
        self.append_submenu(Some(label), model);
    }

    fn widget(&self, name: &str) {
        let item = gio::MenuItem::new(None, None);
        item.set_attribute_value("custom", Some(&name.to_variant()));
        self.append_item(&item);
    }
}

/// Trait that extends [`gio::Settings`] with action safety methods.
pub trait SafeSettings: IsA<gio::Settings> {
    /// Safe version of [`create_action`](gio::prelude::SettingsExt::create_action)
    /// for action safeties with variants or without value nor variants.
    fn create_action_safe<T: DetailableAction>(&self) -> gio::Action {
        self.create_action(T::NAME)
    }

    /// Safe version of [`bind`](gio::prelude::SettingsExtManual::bind)
    /// for setting safeties with variants or without value nor variants.
    fn bind_safe<'a, T: DetailableSetting>(
        &'a self,
        object: &'a impl IsA<glib::Object>,
        property: &'a str,
    ) -> gio::BindingBuilder<'a> {
        self.bind(T::NAME, object, property)
    }

    /// Safe version of [`set`](gio::prelude::SettingsExtManual::set) for stateful setting safeties without value.
    fn set_safe<'a, T: WithoutValue + Stateful<'a>>(
        &self,
        _safety: T,
        state: T::State,
    ) -> Result<(), glib::BoolError> {
        self.set(T::NAME, state.to_variant())
    }

    /// Safe version of [`set_value`](gio::prelude::SettingsExt::set_value) for setting safeties with variants.
    fn set_safe_enum<T>(&self, safety: T) -> Result<(), glib::BoolError>
    where
        T: for<'a> WithValue<'a> + DetailableSetting,
    {
        self.set_value(T::NAME, &safety.to_variant().unwrap()) // NOTE could be unwrap_unchecked()
    }

    /// Safe version of [`get`](gio::prelude::SettingsExtManual::get) for stateful setting safeties without value.
    fn get_safe<'a, T: WithoutValue + Stateful<'a>>(&self, _safety: T) -> T::Owned {
        self.get(T::NAME)
    }

    /// Safe version of [`value`](gio::prelude::SettingsExt::value) for setting safeties with variants.
    fn get_safe_enum<T: for<'a> WithValue<'a> + DetailableSetting>(&self) -> T {
        T::from_variant(&self.value(T::NAME))
    }
}

impl<T: IsA<gio::Settings>> SafeSettings for T {}

/// Trait that extends [`gio::SimpleAction`] with action safety methods.
pub trait SafeSimpleAction: IsA<gio::SimpleAction> {
    /// Safe version of [`new`](gio::SimpleAction::new) for stateless action safeties.
    fn new_safe<T: ActionSafety + Stateless>() -> Self;

    /// Safe version of [`new_stateful`](gio::SimpleAction::new_stateful) for stateful action safeties.
    fn new_stateful_safe<'a, T: ActionSafety + Stateful<'a>>(state: T::State) -> Self;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate) for stateless action safeties without value.
    fn connect_activate_safe<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + WithoutValue + Stateless,
        F: Fn(T, &Self) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// for stateless action safeties with value and without variants.
    fn connect_activate_safe_with_target<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> WithValue<'a> + Stateless + NotDetailable,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// for stateful action safeties without value.
    fn connect_activate_safe_with_state<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + WithoutValue + for<'a> Stateful<'a>,
        F: Fn(T, &Self, <T as Stateful<'_>>::Mapping) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// to conveniently mutate state for stateful action safeties without value.
    fn connect_activate_safe_with_mut_state<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + WithoutValue + for<'a> Stateful<'a>,
        F: Fn(T, &Self, &mut <T as Stateful<'_>>::Owned) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// for stateful action safeties with value and without variants.
    fn connect_activate_safe_with_target_and_state<T, F>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId
    where
        for<'a> T: ActionSafety + WithValue<'a> + Stateful<'a> + NotDetailable,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping, <T as Stateful<'_>>::Mapping) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// to conveniently mutate state for stateful action safeties with value and without variants.
    fn connect_activate_safe_with_target_and_mut_state<T, F>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId
    where
        for<'a> T: ActionSafety + WithValue<'a> + Stateful<'a> + NotDetailable,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping, &mut <T as Stateful<'_>>::Owned) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// for action safeties with variants.
    fn connect_activate_safe_enum<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> WithValue<'a> + DetailableAction,
        F: Fn(&Self, T) + 'static;
}

impl SafeSimpleAction for gio::SimpleAction {
    fn new_safe<T: ActionSafety>() -> Self {
        gio::SimpleAction::new(T::NAME, T::variant_type().as_deref())
    }

    fn new_stateful_safe<'a, T: Stateful<'a>>(state: T::State) -> Self {
        gio::SimpleAction::new_stateful(T::NAME, T::variant_type().as_deref(), state.to_variant())
    }

    fn connect_activate_safe<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety,
        F: Fn(T, &Self) + 'static,
    {
        self.connect_activate(move |this, _| callback(T::SELF, this))
    }

    fn connect_activate_safe_with_target<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> WithValue<'a>,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping) + 'static,
    {
        self.connect_activate(move |this, variant| {
            callback(T::SELF, this, T::map(variant.unwrap()))
        })
    }

    fn connect_activate_safe_with_state<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> Stateful<'a>,
        F: Fn(T, &Self, <T as Stateful<'_>>::Mapping) + 'static,
    {
        self.connect_activate(move |this, _| {
            callback(T::SELF, this, T::map(&this.state().unwrap()))
        })
    }

    fn connect_activate_safe_with_mut_state<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> Stateful<'a>,
        F: Fn(T, &Self, &mut <T as Stateful<'_>>::Owned) + 'static,
    {
        self.connect_activate(move |this, _| {
            let mut state = this.state().unwrap().get().unwrap();
            callback(T::SELF, this, &mut state);
            this.set_state(state.to_variant());
        })
    }

    fn connect_activate_safe_with_target_and_state<T, F>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId
    where
        for<'a> T: ActionSafety + WithValue<'a> + Stateful<'a>,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping, <T as Stateful<'_>>::Mapping) + 'static,
    {
        self.connect_activate(move |this, variant| {
            callback(
                T::SELF,
                this,
                <T as WithValue>::map(variant.unwrap()),
                <T as Stateful>::map(&this.state().unwrap()),
            )
        })
    }

    fn connect_activate_safe_with_target_and_mut_state<T, F>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId
    where
        for<'a> T: ActionSafety + WithValue<'a> + Stateful<'a>,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping, &mut <T as Stateful<'_>>::Owned) + 'static,
    {
        self.connect_activate(move |this, variant| {
            let mut state = this.state().unwrap().get().unwrap();
            callback(
                T::SELF,
                this,
                <T as WithValue>::map(variant.unwrap()),
                &mut state,
            );
            this.set_state(state.to_variant());
        })
    }

    fn connect_activate_safe_enum<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: for<'a> WithValue<'a> + DetailableAction,
        F: Fn(&Self, T) + 'static,
    {
        self.connect_activate(move |this, variant| {
            callback(this, T::from_variant(variant.unwrap()))
        })
    }
}

#[cfg(feature = "libadwaita")]
/// Trait that extends [`adw::Toast`] with action safety methods.
pub trait SafeToast {
    /// Appropriately assigns the [name][n] and [target value][t] of an action
    /// according to an action safety with variants or without value nor variants.
    ///
    /// [n]: adw::Toast::set_action_name
    /// [t]: adw::Toast::set_action_target_value
    fn set_action_safe<T: DetailableAction>(&self, safety: T);

    /// Appropriately assigns the [name][n] and [target value][t] of an
    /// action according to an action safety with value and without variants.
    ///
    /// [n]: adw::Toast::set_action_name
    /// [t]: adw::Toast::set_action_target_value
    fn set_target_safe<'a, T: WithValue<'a>>(&self, _safety: T, target: T::Value)
    where
        T: ActionSafety + NotDetailable;
}

#[cfg(feature = "libadwaita")]
impl SafeToast for adw::Toast {
    fn set_action_safe<T: DetailableAction>(&self, safety: T) {
        // self.set_detailed_action_name(safety.detailed_action_name());
        self.set_action_name(Some(T::FULL_NAME));
        self.set_action_target_value(safety.to_variant().as_ref());
    }

    fn set_target_safe<'a, T: WithValue<'a>>(&self, _safety: T, target: T::Value)
    where
        T: ActionSafety + NotDetailable,
    {
        self.set_action_name(Some(T::FULL_NAME));
        self.set_action_target_value(Some(&target.to_variant()))
    }
}
