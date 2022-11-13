mod has_struct_field;
mod if_branch;
mod property_name;
mod widget;
mod widget_func;

/// The positions or scopes at which widget fields can be generated.
///
/// Typically, widget fields are first generated in init
/// and later destructured in `update_view`.
#[derive(PartialEq, Eq)]
pub(crate) enum WidgetFieldsScope {
    Init,
    ViewUpdate,
}
