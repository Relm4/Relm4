# Changelog

## Unreleased

### Added

+ core: Runtimes of `Component`s and `AsyncComponents` can now be detached for a static lifetime
+ core: Add `ComponentStream` as alternative to `Controller` that implements `Stream` for async message handling
+ core: Added asynchronous components including macro support
+ core: Add `gnome_42` and `gnome_43` feature flags
+ core: Implement `RelmContainerExt` for `adw::Squeezer`
+ core: Implement `RelmSetChildExt` for `gtk::WindowHandle`
+ macros: Auto-generate the name of the `Widgets` type if possible

### Changed

+ core: Add `gnome_42` and `gnome_43` feature flags
+ macros: Allow using methods calls as widget initializers in the `view` macro
+ macros: Explicitly using `visibility` as attribute name is no longer supported

### Fixed

+ all: Fix doc builds on docs.rs and add a CI job to prevent future doc failures
+ core: Fix various bugs related to component shutdown
+ core: `shutdown` on `Component` now works as expected
+ core: `transient_for` on `ComponentBuilder` now works properly when called after the application has been initialized
+ core: `shutdown` on `FactoryComponent` now works as expected
+ macros: Fix type parsing after arrow operator in widget assignments

## 0.5.0-beta.4 - 2022-10-24

### Added

+ core: Added `dox` feature to be able to build the docs without the dependencies
+ core: Added widget templates
+ core: Allow changing the priority of event loops of components
+ core: Impl `ContainerChild` and `RelmSetChildExt` for `adw::ToastOverlay`
+ components: Added `dox` feature to be able to build the docs without the dependencies
+ examples: Add libadwaita Leaflet sidebar example
+ examples: Port entry, actions and popover examples to 0.5

### Changed

+ core: Improved `DrawHandler`
+ core: Made the `macros` feature a default feature
+ core: Remove async-oneshot dependency and replace it with tokio's oneshot channel
+ core: Remove WidgetPlus in favor of RelmWidgetExt
+ core: Add convenience getter-methods to Controller
+ core: `add_action` of `RelmActionGroup` now takes a reference to a `RelmAction` as a parameter
+ examples: Many improvements
+ macros: `parse_with_path`, `update_stream`, `inject_view_code` and `generate_tokens` take references for some of their parameters
+ artwork: Update logo

### Fixed

+ macros: Fix usage of RelmContainerExt with local_ref attribute
+ macros: Report RelmContainerExt error at the correct span

## 0.5.0-beta.3 - 2022-9-28

### Added

+ core: Add `iter_mut` to `FactoryVecDeque`
+ core: Impl extension traits and `FactoryView` for `adw::PreferencesGroup`
+ core: Add a `prelude` module that contains commonly imported traits and types
+ core: Implement RelmContainerExt for Leaflet, Carousel and TabView
+ core: Add `iter()` method to `FactoryVecDeque`
+ core: Add getter for global application to simplify graceful shutdown of applications
+ core: Add MessageBroker type to allow communication between components on different levels
+ core: Return a clone of the `DynamicIndex` after inserting into a factory
+ macros: Add shorthand syntax for simple input messages
+ macros: Add chain attribute for properties
+ components: Add `SimpleComboBox` type as a more idiomatic wrapper around `gtk::ComboBoxText`
+ components: Port `OpenButton` to 0.5
+ book: Many chapters ported to 0.5

### Changed

+ core: Improve `SharedState` interface and prefer method names related to `RwLock`
+ core: Remove Debug requirement for FactoryComponent
+ core: Remove `input` and `output` fields on `ComponentSender` and `FactoryComponentSender` in favor of `input_sender` and `output_sender` methods
+ core: Make `ComponentSender` and `FactoryComponentSender` structs instead of type aliases
+ core: Increase MSRV to 1.63 to match the gtk4 crate
+ core: Rename `ParentMsg` and `output_to_parent_msg` to `ParentInput` and `output_to_parent_input`, respectively.
+ core: Do not call `gtk_init` and `adw_init` in favor of the application startup handler
+ core: Remove `Application` type alias in favor of `gtk::Application`
+ core: Make `app` field on `RelmApp` private
+ core: Use late initialization for transient_for and its native variant
+ core: Rename InitParams to Init in SimpleComponent and Worker too
+ macros: Don't generate dead code in the widgets struct
+ macros: Improve error reporting on invalid trait implementations

### Fixed

+ core: Append children for `gtk::Dialog` to its content area instead of using `set_child`
+ macros: Fix returned widgets assignment in the view macro

### Misc

+ all: Use more clippy lints and clean up the code in general

## 0.5.0-beta.2 - 2022-8-12

### Added

+ core: Add oneshot_command method to ComponentSender
+ core: Implement FactoryView for adw::Carousel
+ components: Complete port to 0.5
+ examples: More examples ported to 0.5

### Changed

+ core: Rename InitParams to Init
+ core: Pass senders by value
+ core: Make factories use FactoryComponentSender instead of individual senders for input and output
+ core: Remove generics from FactoryComponent

### Fixed

+ macros: Fix unsoundness with thread local memory

## 0.5.0-beta.1 - 2022-7-26

### Added

+ core: Introduce commands

### Changed

+ core: The Component trait replaces AppUpdate, ComponentUpdate, AsyncComponentUpdate, MessageHandler, MicroModel, MicroWidgets, Components and Widgets
+ core: Replace FactoryPrototype with FactoryComponent
+ core: Drop FactoryVec and make FactoryVecDeque easier to use
+ core: Improved component initialization and lifecycle
+ macros: Replace iterate, track and watch with attributes
+ macros: Replace args! with only parenthesis
+ macros: Improved macro syntax
+ examples: Many rewrites for the new version

## 0.4.4 - 2022-3-30

### Changed

+ all: Repositories were transferred to the Relm4 organization

### Fixed

+ macros: Don't interpret expr != expr as macro
+ core: Always initialize GTK/Libadwaita before running apps
+ macros: Some doc link fixes

## 0.4.3 - 2022-3-13

### Added

+ core: Add WidgetRef trait to make AsRef easier accessible for widgets
+ macros: Destructure widgets in Widgets::view

### Fixed

+ macros: Use correct widget type in derive macro for components
+ macros: Fix parsing of `property: value == other,` expressions
+ core: Fixed the position type for TabView
+ core: Fixed state changes in FactoryVec (by [V02460](https://github.com/V02460))
+ macros: Parse whole expressions instead of just literals

## 0.4.2 - 2022-2-4

### Added

+ macros: The view macro now allows dereferencing widgets with *

### Fixed

+ core: Fixed clear method of FactoryVec
+ macros: The micro_component macro now parses post_view correctly
+ macros: Fix the ordering of properties in the view macro
+ macros: Fix the ordering of widget assignments in the view macro

## 0.4.1 - 2022-1-17

### Added

+ macros: Improved documentation

### Fixed

+ core: Action macros now include the required traits themselves
+ macros: Allow connecting events in the view macro

## 0.4.0 - 2022-1-16

### Added

+ all: Update gtk4-rs to v0.4
+ core: Introduce the "macro" feature as alternative to using relm4-macros separately
+ macros: Add a macros for MicroComponents and Factories
+ macros: Add a post_view function to execute code after the view code of the macro
+ macros: Allow using the view and menu macros independently from the widget macro
+ macros: Allow using mutable widgets in view
+ macros: Improve error messages for anonymous widgets

### Changed

+ core: Renamed methods of the FactoryPrototype trait to better match with the rest of Relm4
+ macros: manual_view is now called pre_view
+ book: Reworked introduction and first chapter

### Fixed

+ core: Fix panic caused by the clear method of FactoryVecDeque

## 0.4.0-beta.3 - 2021-12-28

### Added

+ core: A factory view implementation for libadwaita's StackView
+ macros: Allow early returns in manual_view (by [euclio](https://github.com/euclio))

### Changed

+ core: Make GTK's command line argument handling optional (by [euclio](https://github.com/euclio))
+ core: DynamicIndex now implements Send but panics when used on other threads

## 0.4.0-beta.2 - 2021-11-26

+ macros: Add optional returned widget syntax

## 0.4.0-beta.1 - 2021-11-21

### Added

+ core: Micro components
+ core: Type safe actions API
+ macros: Menu macro for creating menus
+ macros: New returned widget syntax
+ examples Micro components example

### Changed

+ core: Initialize widgets from the outermost components to the app
+ macros: component! removed and parent! was added instead

### Removed

+ core: RelmComponent::with_new_thread

## 0.2.1 - 2021-10-17

### Added

+ core: Added sender method to RelmComponent
+ macros: New shorthand tracker syntax
+ macros: Allow generic function parameters in properties

### Changed

+ core: Use adw::Application when "libadwaita" feature is active

## 0.2.0 - 2021-10-09

### Changed

+ core: Pass model in connect_components function of the Widgets trait
+ core: Mini rework of factories
+ core: Removed DefaultWidgets trait in favor of Default implementations in gkt4-rs
+ book: Many book improvements by [tronta](https://github.com/tronta)

### Added

+ core: Added with_app method that allows passing an existing gtk::Application to Relm4
+ core: Methods to access the widgets of components
+ core: Re-export for gtk
+ macros: Support named arguments in the widget macro (by [mskorkowski](https://github.com/mskorkowski))
+ macros: Support usage of re-export paths in the widget macro (by [mskorkowski](https://github.com/mskorkowski))
+ macros: Added error message when confusing `=` and `:`
+ macros: Allow usage of visibilities other than pub
+ macros: New pre_connect_components and post_connect_components for manual components code

### Fixed

+ macros: Parsing the first widget should now always work as expected
+ macros: [#20](https://github.com/Relm4/relm4/issues/20) Fix wrong order when using components in the widget macro

## 0.1.0 - 2021-09-06

### Added

+ core: Added message handler type
+ core: More methods for factory data structures
+ macros: Add syntax for connecting events with components
+ examples: Stack example
+ book: Added macro expansion chapter

### Changed

+ book: Added message handler chapter and reworked the threads and async chapter
+ book: Many book improvements by [tronta](https://github.com/tronta)
+ core: The send! macro no longer clones the sender
+ macros: Make fields of public widgets public
+ components: Use &'static str instead of String for configurations
+ examples: Many improvements

### Fixed

+ macros: Use fully qualified syntax for factories
+ macros: Passing additional arguments now works for components and other properties, too.

## 0.1.0-beta.9 - 2021-08-24

### Added

+ components: Open button with automatic recent files list
+ components: Removed trait duplication and added more docs
+ core: Iterators added to factory data structures
+ core: More widgets added as FactoryView

### Changed

+ book: Many book improvements by [tronta](https://github.com/tronta)
+ core: Removed class name methods from WidgetPlus [#14](https://github.com/Relm4/relm4/pull/14)

### Fixed

+ macros: Parsing additional fields should be more stable now
+ macros: Widgets can not include comments at the top

## 0.1.0-beta.8 - 2021-08-20

### Added

+ core: Support for libadwaita 🎉
+ macros: Fully qualified syntax for trait disambiguation
+ macros: Allow passing additional arguments to widget initialization (useful e.g. for grids)
+ book: Reusable components and widget macro reference chapters

### Changed

+ macros: Improved error messages

## 0.1.0-beta.7 - 2021-08-19

### Added

+ book: Factory, components, worker and thread + async chapters

### Changed

+ core: get and get_mut of FactoryVec and FactoryVecDeque now return an Option to prevent panics

### Fixed

+ macros: Fixed components
+ core: Fixed unsound removal of elements in FactoryVecDeque


## 0.1.0-beta.6 - 2021-08-18

### Changed

+ core: Improved and adjusted the FactoryPrototype trait

### Added

+ core: Added the FactoryListView trait for more flexibility
+ core: Added a FactoryVecDeque container
+ core: Implemented FactoryView and FactoryListView for more widgets
+ examples: More examples

### Fixed

+ macros: Fixed the factory! macro

## 0.1.0-beta.5 - 2021-08-15

### Added

+ core: Drawing handler for gtk::DrawingArea
+ core: New CSS methods in WidgetPlus trait
+ examples: Many new examples

### Changed

+ core: Many doc improvements
+ macros: Improved tracker! macro
