//! Commonly-imported traits and types.
//!
//! Modules that contain components can glob import this module to bring all needed types and
//! traits into scope.

pub use crate::factory::{DynamicIndex, FactoryComponent, FactoryComponentSender};
pub use crate::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    SimpleComponent, WidgetPlus,
};
#[cfg(feature = "libadwaita")]
pub use adw;
pub use gtk;
#[cfg(feature = "libpanel")]
pub use panel;
