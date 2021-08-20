# Changelog

## Unreleased

### Added

+ Fully qualified syntax for trait disabiguation in relm4-macros
+ Support for libadwaita 🎉
+ Allow passing additional arguments to widget initialization (useful e.g. for girds)
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
