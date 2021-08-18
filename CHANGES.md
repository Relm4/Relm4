# Changelog

## Unreleased

### Added

+ Book: Factory chapters

### Changed

+ get and get_mut of FactoryVec and FactoryVecDeque now return an Option to prevent panics

### Fixed

+ Fix unsound removal of elements in FactoryVecDeque


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
