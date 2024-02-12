/*
 * SPDX-FileCopyrightText: 2023 Eduardo Javier Alvarado Aarón <eduardo.javier.alvarado.aaron@gmail.com>
 *
 * SPDX-License-Identifier: (Apache-2.0 or MIT)
 */

//! Utilities to safely manage actions and settings.

pub mod extensions;

use gtk::glib;

/// The basic trait for declaring setting safeties.
pub trait SettingSafety {
    /// The name of the setting or action.
    ///
    /// The macro [`safe_settings_and_actions!`](crate::safe_settings_and_actions!)
    /// automatically implements [`Display`](std::fmt::Display) using this value,
    /// so it is not necessary to use it directly for logging and similar cases.
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
    type Value: gtk::prelude::ToVariant + Into<glib::Variant>;

    /// Type used to receive the value on signals.
    type Mapping;

    /// Function to convert [`struct@glib::Variant`] to the [`Mapping`](WithValue::Mapping) type.
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
    type State: gtk::prelude::ToVariant;

    /// Owned version of the [`State`](Stateful::State) type, used by getters or methods that mutate state.
    type Owned: gtk::prelude::ToVariant + gtk::prelude::FromVariant;

    /// Type used to receive the state on signals.
    type Mapping;

    /// Function to convert [`struct@glib::Variant`] to the [`Mapping`](Stateful::Mapping) type.
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
/// relm4::safe_settings_and_actions! {
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
/// See also: [`safe_settings_and_actions!`](crate::safe_settings_and_actions!), [`NotDetailable`].
pub trait DetailableSetting: SettingSafety {
    /// Function to convert [`struct@glib::Variant`] to `Self`, especially when `Self` is an `enum`.
    fn from_variant(variant: &glib::Variant) -> Self
    where
        Self: for<'a> WithValue<'a>;

    /// Function to convert `Self` to [`struct@glib::Variant`], especially when `Self` is an `enum`.
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
/// relm4::safe_settings_and_actions! {
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
/// See also: [`safe_settings_and_actions!`](crate::safe_settings_and_actions!), [`NotDetailable`].
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
/// 3. `map`: a method of [`struct@glib::Variant`] to convert it to the correct type data, used to receive the
///           `@value` or `@state` in signals, or to retrieve an enumerated variant from [`struct@glib::Variant`].
///
/// Unlike `param` and `owned`, `map` is specified as follows: `<method> ReturnType`.
/// If not specified, it falls back to: `<get> OwnedType`.
/// If only the method and not the return type is specified, falls back to: `<method> ParamType`.
///
/// In the case of `@value` it is not necessary `owned` if `map` is specified.
///
/// ## Example
/// ~~~
/// use relm4::gtk::prelude::{StaticVariantType, ToVariant};
///
/// relm4::safe_settings_and_actions! {
///     // Action safeties must specify the group parameter.
///     UnitStruct(group: "action-group", name: "action-or-setting-name");
///
///     // If a lifetime is necessary, use 'a.
///     @value(param: &'a str, owned: String, map: <str> &'a str)
///     @state(param: &'a str, owned: String, map: <str> &'a str)
///     pub Enum(name: "setting-name") {
///         Variant = ("literal-value"), // ... Variants require @value.
///         // In the case of string literals, they must always be enclosed
///         // in parens for keyboard accelerators and settings to work.
///     }
/// }
/// ~~~
macro_rules! safe_settings_and_actions {
    (fallback! @($($foo:tt)+)            ) => { $($foo)+ };
    (fallback! @($($foo:tt)+) $($bar:tt)+) => { $($bar)+ };

    ( value:     $value:literal      ) => { $value };
    ( value: (   $value:literal     )) => { $value };
    ( value: [ $($value:tt),+ $(,)? ]) => { &[$($crate::safe_settings_and_actions!(value: $value)),+] };
    ( value: ( $($value:tt),+ $(,)? )) => {  ($($crate::safe_settings_and_actions!(value: $value)),+) };
    (detail:     $value:literal      ) => { $value };
    (detail: (   $value:literal     )) => { concat!("'", $value, "'") };
    (detail: [ $($value:tt),+ $(,)? ]) => { concat!("[" $(,$crate::safe_settings_and_actions!(detail: $value),)','+ "]") };
    (detail: ( $($value:tt),+ $(,)? )) => { concat!("(" $(,$crate::safe_settings_and_actions!(detail: $value),)','+ ")") };

    (setting: $name:literal;) => {
        fn from_variant(_: &$crate::gtk::glib::Variant) -> Self { Self }
        fn to_variant(&self) -> Option<$crate::gtk::glib::Variant> { None }
    };
    (action: $group:literal $name:literal;) => {
        fn detailed_action_name(&self) -> &'static str {
            <Self as $crate::safe_settings_and_actions::ActionSafety>::FULL_NAME
        }
    };
    (setting: $name:literal { $($variant:ident = $value:tt),+ $(,)? }) => {
        fn from_variant(variant: &$crate::gtk::glib::Variant) -> Self {
            match <Self as $crate::safe_settings_and_actions::WithValue>::map(&variant) {
                $($crate::safe_settings_and_actions!(value: $value) => Self::$variant,)+
                _ => unreachable!()
            }
        }
        fn to_variant(&self) -> Option<$crate::gtk::glib::Variant> {
            Some(match self {
                $(Self::$variant => $crate::safe_settings_and_actions!(value: $value).to_variant()),+
            })
        }
    };
    (action: $group:literal $name:literal { $($variant:ident = $value:tt),+ $(,)? }) => {
        fn detailed_action_name(&self) -> &'static str {
            match self {
                $(Self::$variant => concat!($group, '.', $name, '(', $crate::safe_settings_and_actions!(detail: $value), ')')),+
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
        $crate::safe_settings_and_actions!(self_type! $(#[$attr])* $vis $type $variants);

        impl $crate::safe_settings_and_actions::SettingSafety for $type {
            const NAME: &'static str = $name;
            fn variant_type<'a>() -> Option<std::borrow::Cow<'static, $crate::gtk::glib::VariantTy>> {
                $crate::safe_settings_and_actions!(fallback! @(None) $(Some(<$target_param>::static_variant_type()))?)
            }
        }

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", <$type as $crate::safe_settings_and_actions::SettingSafety>::NAME)
            }
        }

        $(impl $crate::safe_settings_and_actions::ActionSafety for $type {
            const FULL_NAME: &'static str = concat!($group, '.', $name);
            const SELF: Self = $crate::safe_settings_and_actions!(self_constant! $type $variants);
        })?

        $crate::safe_settings_and_actions! { fallback!
            @(impl $crate::safe_settings_and_actions::WithoutValue for $type { })
            $(impl<'a> $crate::safe_settings_and_actions::WithValue<'a> for $type {
                type Value = $target_param;
                type Mapping = $crate::safe_settings_and_actions!(fallback! @(
                    $crate::safe_settings_and_actions!(fallback! @($target_param) $($target_owned)?)
                ) $($($target_mapping)?)?);

                $crate::safe_settings_and_actions! { fallback!
                    @(fn map(variant: &'a $crate::gtk::glib::Variant) -> $crate::safe_settings_and_actions! {
                        fallback! @($target_param) $($target_owned)?
                    } { variant.get().unwrap() })

                    $(fn map(variant: &'a $crate::gtk::glib::Variant) -> Self::Mapping {
                        variant.$target_map().unwrap()
                    })?
                }
            })?
        }

        $crate::safe_settings_and_actions! { fallback!
            @(impl $crate::safe_settings_and_actions::Stateless for $type { })
            $(impl<'a> $crate::safe_settings_and_actions::Stateful<'a> for $type {
                type State   = $state_param;
                type Owned   = $crate::safe_settings_and_actions!(fallback! @($state_param) $($state_owned)?);
                type Mapping = $crate::safe_settings_and_actions!(fallback! @(Self::Owned) $($state_mapping)?);

                $crate::safe_settings_and_actions! { fallback!
                    @(fn map(variant: &'a $crate::gtk::glib::Variant) -> $crate::safe_settings_and_actions! {
                        fallback! @($state_param) $($state_owned)?
                    } { variant.get().unwrap() })

                    $(fn map(variant: &'a $crate::gtk::glib::Variant) -> Self::Mapping {
                        variant.$state_map().unwrap()
                    })?
                }
            })?
        }

        $crate::safe_settings_and_actions! { detailable_or_not!
            $type [$($target_param)?] [$variants]
            impl $crate::safe_settings_and_actions::DetailableSetting for $type {
                $crate::safe_settings_and_actions!(setting: $name $variants);
            }
            $(impl $crate::safe_settings_and_actions::DetailableAction for $type {
                $crate::safe_settings_and_actions!(action: $group $name $variants);
            })?
            #[allow(dead_code)]
            impl $type {
                $crate::safe_settings_and_actions!(value! $(
                    $crate::safe_settings_and_actions!(fallback! @($target_param) $($($target_mapping)?)?)
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

    (detailable_or_not! $type:ty [$target:ty] [         ;  ] $($impl:tt)+) => { impl $crate::safe_settings_and_actions::NotDetailable for $type { } };
    (detailable_or_not! $type:ty [          ] [         ;  ] $($impl:tt)+) => { $($impl)+ };
    (detailable_or_not! $type:ty [$target:ty] [$variants:tt] $($impl:tt)+) => { $($impl)+ };

    (value! ;) => { };
    (value! $target:ty { $($variant:ident = $value:tt),+ $(,)? }) => {
        const fn value<'a>(&self) -> $target {
            match self { $(Self::$variant => $crate::safe_settings_and_actions!(value: $value)),+ }
        }
        const fn some<'a>(&self) -> Option<$target> {
            Some(match self { $(Self::$variant => $crate::safe_settings_and_actions!(value: $value)),+ })
        }
    };

    ($($(#[$attr:meta])* $(@$keyword:tt $params:tt)* $vis:vis $type:ident $naming:tt $variants:tt)*) => {
        $($crate::safe_settings_and_actions!(safety! $(#[$attr])* $(@$keyword $params)* $vis $type $naming $variants);)*
    };
}
