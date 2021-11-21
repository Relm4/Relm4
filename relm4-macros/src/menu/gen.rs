use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, Path};

use super::{Menu, MenuEntry, MenuItem, MenuSection, Menus};

impl Menus {
    pub fn menus_stream(&self, relm4_path: &Path) -> TokenStream2 {
        let mut menu_stream = TokenStream2::new();

        for item in &self.items {
            menu_stream.extend(item.menu_stream(relm4_path));
        }

        menu_stream
    }
}

impl Menu {
    fn menu_stream(&self, relm4_path: &Path) -> TokenStream2 {
        let name = &self.name;
        let mut menu_stream = quote! {
            let #name = #relm4_path::gtk::gio::Menu::new();
        };

        for item in &self.items {
            menu_stream.extend(item.item_stream(name, relm4_path));
        }

        menu_stream
    }
}

impl MenuItem {
    fn item_stream(&self, parent_ident: &Ident, relm4_path: &Path) -> TokenStream2 {
        let mut item_stream = TokenStream2::new();

        match self {
            MenuItem::Entry(entry) => {
                item_stream.extend(entry.entry_stream(parent_ident, relm4_path))
            }
            MenuItem::Section(section) => {
                item_stream.extend(section.section_stream(parent_ident, relm4_path))
            }
        }

        item_stream
    }
}

impl MenuEntry {
    fn entry_stream(&self, parent_ident: &Ident, relm4_path: &Path) -> TokenStream2 {
        let string = &self.string;
        let ty = &self.action_ty;
        if let Some(value) = &self.value {
            quote! {
                let new_entry = #relm4_path::gtk::gio::MenuItem::new(Some(#string), Some(& #ty::action_name()));
                new_entry.set_action_and_target_value(Some(&#ty::action_name()), Some(&#relm4_path::gtk::glib::variant::ToVariant::to_variant(&#value)));
                #parent_ident.append_item(&new_entry);
            }
        } else {
            quote! {
                let new_entry = #relm4_path::gtk::gio::MenuItem::new(Some(#string), Some(& #ty::action_name()));
                #parent_ident.append_item(&new_entry);
            }
        }
    }
}

impl MenuSection {
    fn section_stream(&self, parent_ident: &Ident, relm4_path: &Path) -> TokenStream2 {
        let name = &self.name;
        let mut section_stream = quote! {
            let #name = #relm4_path::gtk::gio::Menu::new();
            #parent_ident.append_section(None, &#name);
        };

        for item in &self.items {
            section_stream.extend(item.item_stream(name, relm4_path));
        }

        section_stream
    }
}
