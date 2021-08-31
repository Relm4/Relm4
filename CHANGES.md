# Changelog

## Unreleased

### Added

+ relm4: More methods for factory data structures
+ relm4-examples: Stack example
+ book: Added macro expansion chapter

### Changed

+ book: Many book improvements by [tronta](https://github.com/tronta)
+ relm4-macros: Make fields of public widgets public
+ relm4-components: Use &'static str instead of String for configurations
+ relm4-examples: Many improvements

### Fixed

+ relm4-macros: Passing additional arguments now works for components and other properties, too.

## 0.1.0-beta.9

## Added

+ relm4-components: Open button with automatic recent files list
+ relm4-components: Removed trait duplication and added more docs
+ relm4: Iterators added to factory data structures
+ relm4: More widgets added as FactoryView

### Changed

+ book: Many book improvements by [tronta](https://github.com/tronta)
+ relm4: Removed class name methods from WidgetPlus [#14](https://github.com/AaronErhardt/relm4/pull/14)

### Fixed

+ relm4-macros: Parsing additional fields should be more stable now
+ relm4-macros: Widgets can not include comments at the top 

## 0.1.0-beta.8 - 2021-08-20

### Added

+ relm4: Support for libadwaita 🎉
+ relm4-macros: Fully qualified syntax for trait disabiguation
+ relm4-macros: Allow passing additional arguments to widget initialization (useful e.g. for grids)
+ book: Reusable components and widget macro reference chapters

### Changed

+ relm4-macros: Improved error messages

## 0.1.0-beta.7 - 2021-08-19

### Added

+ book: Factory, components, worker and thread + async chapters

### Changed

+ relm4: get and get_mut of FactoryVec and FactoryVecDeque now return an Option to prevent panics

### Fixed

+ relm4-macros: Fixed components
+ relm4: Fixed unsound removal of elements in FactoryVecDeque


## 0.1.0-beta.6 - 2021-08-18

### Changed

+ relm4: Improved and adjusted the FactoryPrototype trait

### Added 

+ relm4: Added the FactoryListView trait for more flexibility
+ relm4: Added a FactoryVecDeque container
+ relm4: Implemented FactoryView and FactoryListView for more widgets
+ relm4-examples: More examples

### Fixed

+ relm4-macros: Fixed the factory! macro

## 0.1.0-beta.5 - 2021-08-15

### Added

+ relm4: Drawing handler for gtk::DrawingArea
+ relm4: New CSS methods in WidgetPlus trait
+ relm4-examples: Many new examples

### Changed

+ relm4: Many doc improvements
+ relm4-macros: Improved tracker! macro
