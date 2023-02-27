use crate::{
    factory::{
        positions::{GridPosition, StackPageInfo},
        FactoryView,
    },
    gtk, RelmIterChildrenExt, WidgetRef,
};
use gtk::prelude::{FlowBoxChildExt, ListBoxRowExt};

/// Assert the exact ordering of widget children. The container must implement `RelmIterChildrenExt`.
macro_rules! assert_children {
    ($container:ident: None) => {
        if let Some(child) = $container.iter_children().next() {
            panic!("Container expected be empty, but there is at least one child: {child:?}");
        }
    };
    ($container:ident: $($child:ident),*) => {
        {
            let mut actual_children = $container.iter_children();
            let mut index = 0;
            $(
                let actual = actual_children.next();
                let expected = &$child;
                if  actual.as_ref().map(|widget| widget.widget_ref()) != Some(expected.widget_ref()) {
                    panic!(
                        "Children at index {index} does not match; expected: Some({expected:?}), actual: {actual:?}"
                    )
                }
                index += 1;
            )+
            if let Some(actual) = actual_children.next() {
                panic!("Children at index {index} does not match; expected: None, actual: {actual:?}");
            }
        }
    };
}

#[gtk::test]
fn box_factory_view() {
    let gtk_box = gtk::Box::default();

    let widget1 = gtk::Label::default();
    let widget2 = gtk::Switch::default();
    let widget3 = gtk::Entry::default();

    let w2 = gtk_box.factory_append(&widget2, &());
    let w3 = gtk_box.factory_insert_after(&widget3, &(), &w2);
    let w1 = gtk_box.factory_prepend(&widget1, &());

    assert_eq!(widget1, w1);
    assert_eq!(widget2, w2);
    assert_eq!(widget3, w3);

    assert_eq!(gtk::Box::returned_widget_to_child(&w1), widget1);
    assert_eq!(gtk::Box::returned_widget_to_child(&w2), widget2);
    assert_eq!(gtk::Box::returned_widget_to_child(&w3), widget3);

    assert_children!(gtk_box: w1, w2, w3);

    gtk_box.factory_move_after(&w3, &w1);
    assert_children!(gtk_box: w1, w3, w2);

    gtk_box.factory_move_after(&w1, &w2);
    assert_children!(gtk_box: w3, w2, w1);

    gtk_box.factory_move_start(&w2);
    assert_children!(gtk_box: w2, w3, w1);

    gtk_box.factory_move_start(&w1);
    assert_children!(gtk_box: w1, w2, w3);

    gtk_box.factory_remove(&w3);
    assert_children!(gtk_box: w1, w2);

    gtk_box.factory_remove(&w2);
    assert_children!(gtk_box: w1);

    gtk_box.factory_remove(&w1);
    assert_children!(gtk_box: None);
}

#[gtk::test]
fn grid_factory_view() {
    let grid = gtk::Grid::default();

    let widget1 = gtk::Label::default();
    let widget2 = gtk::Switch::default();
    let widget3 = gtk::Entry::default();

    let pos1 = GridPosition {
        column: 0,
        row: 0,
        width: 1,
        height: 1,
    };
    let pos2 = GridPosition {
        column: 1,
        row: 0,
        width: 1,
        height: 2,
    };
    let pos3 = GridPosition {
        column: 0,
        row: 1,
        width: 1,
        height: 1,
    };

    let w2 = grid.factory_append(&widget2, &pos2);
    let w3 = grid.factory_append(&widget3, &pos3);
    let w1 = grid.factory_append(&widget1, &pos1);

    assert_eq!(widget1, w1);
    assert_eq!(widget2, w2);
    assert_eq!(widget3, w3);

    assert_eq!(gtk::Grid::returned_widget_to_child(&w1), widget1);
    assert_eq!(gtk::Grid::returned_widget_to_child(&w2), widget2);
    assert_eq!(gtk::Grid::returned_widget_to_child(&w3), widget3);

    assert_children!(grid: w1, w2, w3);

    grid.factory_remove(&w3);
    assert_children!(grid: w1, w2);

    grid.factory_remove(&w2);
    assert_children!(grid: w1);

    grid.factory_remove(&w1);
    assert_children!(grid: None);
}

#[gtk::test]
fn stack_factory_view() {
    let stack = gtk::Stack::default();

    let widget1 = gtk::Label::default();
    let widget2 = gtk::Switch::default();
    let widget3 = gtk::Entry::default();

    let page1 = stack.factory_append(&widget1, &StackPageInfo::default());
    let page2 = stack.factory_append(&widget2, &StackPageInfo::default());
    let page3 = stack.factory_append(&widget3, &StackPageInfo::default());

    assert_eq!(page1.child(), widget1);
    assert_eq!(page2.child(), widget2);
    assert_eq!(page3.child(), widget3);

    assert_eq!(gtk::Stack::returned_widget_to_child(&page1), widget1);
    assert_eq!(gtk::Stack::returned_widget_to_child(&page2), widget2);
    assert_eq!(gtk::Stack::returned_widget_to_child(&page3), widget3);

    assert_children!(stack: widget1, widget2, widget3);

    stack.factory_remove(&page2);
    assert_children!(stack: widget1, widget3);

    stack.factory_remove(&page3);
    assert_children!(stack: widget1);

    stack.factory_remove(&page1);
    assert_children!(stack: None);
}

#[gtk::test]
fn list_box_factory_view() {
    let list_box = gtk::ListBox::default();

    let widget1 = gtk::Label::default();
    let widget2 = gtk::ListBoxRow::default();
    let widget3 = gtk::Entry::default();

    let row2 = list_box.factory_append(&widget2, &());
    let row3 = list_box.factory_insert_after(&widget3, &(), &row2);
    let row1 = list_box.factory_prepend(&widget1, &());

    assert_eq!(row1.child().as_ref(), Some(widget1.as_ref()));
    assert_eq!(row2, widget2);
    assert_eq!(row3.child().as_ref(), Some(widget3.as_ref()));

    assert_eq!(gtk::ListBox::returned_widget_to_child(&row1), widget1);
    assert_eq!(gtk::ListBox::returned_widget_to_child(&row2), widget2);
    assert_eq!(gtk::ListBox::returned_widget_to_child(&row3), widget3);

    assert_children!(list_box: row1, row2, row3);

    list_box.factory_move_after(&row3, &row1);
    assert_children!(list_box: row1, row3, row2);

    list_box.factory_move_after(&row1, &row2);
    assert_children!(list_box: row3, row2, row1);

    list_box.factory_move_start(&row2);
    assert_children!(list_box: row2, row3, row1);

    list_box.factory_move_start(&row1);
    assert_children!(list_box: row1, row2, row3);

    list_box.factory_remove(&row3);
    assert_children!(list_box: row1, row2);

    list_box.factory_remove(&row2);
    assert_children!(list_box: row1);

    list_box.factory_remove(&row1);
    assert_children!(list_box: None);
}

#[gtk::test]
fn flow_box_factory_view() {
    let flow_box = gtk::FlowBox::default();

    let widget1 = gtk::Label::default();
    let widget2 = gtk::FlowBoxChild::default();
    let widget3 = gtk::Entry::default();

    let child2 = flow_box.factory_append(&widget2, &());
    let child3 = flow_box.factory_insert_after(&widget3, &(), &child2);
    let child1 = flow_box.factory_prepend(&widget1, &());

    assert_eq!(child1.child().as_ref(), Some(widget1.as_ref()));
    assert_eq!(child2, widget2);
    assert_eq!(child3.child().as_ref(), Some(widget3.as_ref()));

    assert_eq!(gtk::FlowBox::returned_widget_to_child(&child1), widget1);
    assert_eq!(gtk::FlowBox::returned_widget_to_child(&child2), widget2);
    assert_eq!(gtk::FlowBox::returned_widget_to_child(&child3), widget3);

    assert_children!(flow_box: child1, child2, child3);

    flow_box.factory_move_after(&child3, &child1);
    assert_children!(flow_box: child1, child3, child2);

    flow_box.factory_move_after(&child1, &child2);
    assert_children!(flow_box: child3, child2, child1);

    flow_box.factory_move_start(&child2);
    assert_children!(flow_box: child2, child3, child1);

    flow_box.factory_move_start(&child1);
    assert_children!(flow_box: child1, child2, child3);

    flow_box.factory_remove(&child3);
    assert_children!(flow_box: child1, child2);

    flow_box.factory_remove(&child2);
    assert_children!(flow_box: child1);

    flow_box.factory_remove(&child1);
    assert_children!(flow_box: None);
}

#[gtk::test]
#[cfg(feature = "libadwaita")]
fn tab_view_factory_view() {
    let tab_view = adw::TabView::default();

    let widget1 = gtk::Label::default();
    let widget2 = gtk::Switch::default();
    let widget3 = gtk::Entry::default();

    let page2 = tab_view.factory_append(&widget2, &());
    let page3 = tab_view.factory_insert_after(&widget3, &(), &page2);
    let page1 = tab_view.factory_prepend(&widget1, &());

    assert_eq!(page1.child(), widget1);
    assert_eq!(page2.child(), widget2);
    assert_eq!(page3.child(), widget3);

    assert_eq!(adw::TabView::returned_widget_to_child(&page1), widget1);
    assert_eq!(adw::TabView::returned_widget_to_child(&page2), widget2);
    assert_eq!(adw::TabView::returned_widget_to_child(&page3), widget3);

    assert_children!(tab_view: widget1, widget2, widget3);

    tab_view.factory_move_after(&page3, &page1);
    assert_children!(tab_view: widget1, widget3, widget2);

    tab_view.factory_move_after(&page1, &page2);
    assert_children!(tab_view: widget3, widget2, widget1);

    tab_view.factory_move_start(&page2);
    assert_children!(tab_view: widget2, widget3, widget1);
}
