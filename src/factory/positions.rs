//! Position types for various widgets.

/// Storing information about where new widgets can be placed
/// inside a [`gtk::Grid`].
#[derive(Debug)]
pub struct GridPosition {
    /// The number of the column.
    pub column: i32,
    /// The number of the row.
    pub row: i32,
    /// The amount of columns the widget should take.
    pub width: i32,
    /// The amount of rows the widget should take.
    pub height: i32,
}

#[derive(Debug)]
/// Position used for [`gtk::Fixed`].
pub struct FixedPosition {
    /// Position on the x-axis.
    pub x: f64,
    /// Position on the y-axis.
    pub y: f64,
}
