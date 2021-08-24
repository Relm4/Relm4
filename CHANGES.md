# Changelog

## Unreleased

## Added

+ relm4-components: Open button with automatic recent files list
+ relm4-components: Removed trait duplication and added more docs
+ relm4: Iterators added to factory data structures
+ relm4: More widgets added as FactoryView

### Changed

+ Many book improvements by [tronta](https://github.com/tronta)
+ relm4: Removed class name methods from WidgetPlus [#14](https://github.com/AaronErhardt/relm4/pull/14)

### Fixed

+ relm4-macros: parsing additional fields should be more stable now
+ relm4-macros: widgets can not include comments at the top 

## 0.1.0-beta.8 - 2021-08-20

### Added

+ Fully qualified syntax for trait disabiguation in relm4-macros
+ Support for libadwaita 🎉
+ Allow passing additional arguments to widget initialization (useful e.g. for grids)
+ Book: Reusable components and widget macro reference chapters

### Changed

+ Improved error messages in relm4-macros

## 0.1.0-beta.7 - 2021-08-19

### Added

+ Book: Factory, components, worker and thread + async chapters

### Changed

+ get and get_mut of FactoryVec and FactoryVecDeque now return an Option to prevent panics

### Fixed

+ Fixed components macro
+ Fixed unsound removal of elements in FactoryVecDeque


## 0.1.0-beta.6 - 2021-08-18

### Changed

+ Improved and adjusted the FactoryPrototype trait

### Added 

+ Added the FactoryListView trait for more flexibility
+ Added a FactoryVecDeque container
+ Implemented FactoryView and FactoryListView for more widgets
+ More examples

### Fixed

+ Fixed the factory! macro in relm4-macros
