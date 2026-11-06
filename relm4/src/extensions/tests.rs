use crate::{RelmIterChildrenExt, RelmListBoxExt, RelmRemoveAllExt};
use gtk::prelude::{BoxExt, GridExt, WidgetExt};

// A set of widgets for tests
#[derive(Default)]
struct TestWidgets(gtk::Label, gtk::Switch, gtk::Box);

impl TestWidgets {
    fn assert_parent(&self) {
        assert!(self.0.parent().is_some());
        assert!(self.1.parent().is_some());
        assert!(self.2.parent().is_some());
    }
}

// Returns whether two optional widgets are the same.
fn same_widgets(
    first: Option<impl AsRef<gtk::Widget>>,
    second: Option<impl AsRef<gtk::Widget>>,
) -> bool {
    match (first, second) {
        (Some(first), Some(second)) => first.as_ref() == second.as_ref(),
        (None, None) => true,
        _ => false,
    }
}

#[gtk::test]
fn box_extension_traits() {
    let gtk_box = gtk::Box::default();
    let widgets = TestWidgets::default();

    gtk_box.append(&widgets.1);
    gtk_box.prepend(&widgets.0);
    gtk_box.append(&widgets.2);

    widgets.assert_parent();

    let mut children = gtk_box.iter_children();
    assert!(same_widgets(children.next(), Some(&widgets.0)));
    assert!(same_widgets(children.next_back(), Some(&widgets.2)));
    assert!(same_widgets(children.next(), Some(&widgets.1)));
    assert_eq!(children.next(), None);
    assert_eq!(children.next_back(), None);

    gtk_box.remove_all();

    assert_eq!(gtk_box.iter_children().next(), None);
}

#[gtk::test]
fn list_box_extension_traits() {
    let list_box = gtk::ListBox::default();
    let widgets = TestWidgets::default();

    list_box.append(&widgets.1);
    list_box.prepend(&widgets.0);
    list_box.append(&widgets.2);

    widgets.assert_parent();

    assert_eq!(list_box.index_of_child(&widgets.0), Some(0));
    assert_eq!(list_box.index_of_child(&widgets.1), Some(1));
    assert_eq!(list_box.index_of_child(&widgets.2), Some(2));

    let mut rows = list_box.iter_children();
    assert!(same_widgets(rows.next_back(), widgets.2.parent()));
    assert!(same_widgets(rows.next(), widgets.0.parent()));
    assert!(same_widgets(rows.next(), widgets.1.parent()));
    assert_eq!(rows.next(), None);
    assert_eq!(rows.next_back(), None);

    list_box.remove_all();
    let _ = &widgets.0.unparent();
    let _ = &widgets.1.unparent();
    let _ = &widgets.2.unparent();

    assert_eq!(list_box.iter_children().next(), None);

    list_box.append(&widgets.0);
    list_box.append(&widgets.1);
    list_box.append(&widgets.2);

    widgets.assert_parent();

    list_box.remove_row_of_child(&widgets.0);
    list_box.remove_row_of_child(&widgets.1);
    list_box.remove_row_of_child(&widgets.2);

    assert_eq!(list_box.iter_children().next(), None);

    assert_eq!(list_box.index_of_child(&widgets.0), None);
    assert_eq!(list_box.index_of_child(&widgets.1), None);
    assert_eq!(list_box.index_of_child(&widgets.2), None);
}

#[gtk::test]
fn flow_box_extension_traits() {
    let flow_box = gtk::FlowBox::default();
    let widgets = TestWidgets::default();

    flow_box.insert(&widgets.1, -1);
    flow_box.insert(&widgets.0, 0);
    flow_box.insert(&widgets.2, -1);

    widgets.assert_parent();

    let mut flow_children = flow_box.iter_children();
    assert!(same_widgets(flow_children.next(), widgets.0.parent()));
    assert!(same_widgets(flow_children.next(), widgets.1.parent()));
    assert!(same_widgets(flow_children.next(), widgets.2.parent()));
    assert_eq!(flow_children.next_back(), None);
    assert_eq!(flow_children.next(), None);

    flow_box.remove_all();

    assert_eq!(flow_box.iter_children().next(), None);
}

#[gtk::test]
fn grid_extension_traits() {
    let grid = gtk::Grid::default();
    let widgets = TestWidgets::default();

    grid.attach(&widgets.0, 0, 0, 1, 1);
    grid.attach(&widgets.2, 2, 2, 2, 2);
    grid.attach(&widgets.1, 1, 0, 1, 1);

    widgets.assert_parent();

    let mut children = grid.iter_children();
    assert!(same_widgets(children.next(), Some(&widgets.0)));
    assert!(same_widgets(children.next(), Some(&widgets.1)));
    assert!(same_widgets(children.next(), Some(&widgets.2)));
    assert_eq!(children.next(), None);
    assert_eq!(children.next_back(), None);

    grid.remove_all();

    assert!(widgets.0.parent().is_none());
    assert!(widgets.1.parent().is_none());
    assert!(widgets.2.parent().is_none());
}

#[gtk::test]
fn stack_extension_traits() {
    let stack = gtk::Stack::default();
    let widgets = TestWidgets::default();

    stack.add_child(&widgets.0);
    stack.add_child(&widgets.1);
    stack.add_child(&widgets.2);

    widgets.assert_parent();

    let mut children = stack.iter_children();
    assert!(same_widgets(children.next(), Some(&widgets.0)));
    assert!(same_widgets(children.next(), Some(&widgets.1)));
    assert!(same_widgets(children.next(), Some(&widgets.2)));
    assert_eq!(children.next_back(), None);
    assert_eq!(children.next(), None);

    stack.remove_all();

    assert_eq!(stack.iter_children().next(), None);
}
