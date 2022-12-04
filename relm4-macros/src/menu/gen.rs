use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Ident, LitStr};

use super::{Menu, MenuEntry, MenuItem, MenuSection, Menus};

impl Menus {
    pub(crate) fn menus_stream(&self) -> TokenStream2 {
        let mut menu_stream = TokenStream2::new();

        for item in &self.items {
            menu_stream.extend(item.menu_stream());
        }

        menu_stream
    }
}

impl Menu {
    fn menu_stream(&self) -> TokenStream2 {
        let name = &self.name;
        let gtk_import = crate::gtk_import();
        let mut menu_stream = quote_spanned! {
            name.span() =>
                let #name = #gtk_import ::gio::Menu::new();
        };

        for item in &self.items {
            menu_stream.extend(item.item_stream(name));
        }

        menu_stream
    }
}

impl MenuItem {
    fn item_stream(&self, parent_ident: &Ident) -> TokenStream2 {
        let mut item_stream = TokenStream2::new();

        item_stream.extend(match self {
            MenuItem::Entry(entry) => entry.entry_stream(parent_ident),
            MenuItem::Section(section) => section.section_stream(parent_ident),
            MenuItem::Custom(id) => custom_stream(parent_ident, id),
        });

        item_stream
    }
}

fn custom_stream(parent_ident: &Ident, id: &LitStr) -> TokenStream2 {
    let gtk_import = crate::gtk_import();
    quote_spanned! {
        id.span() =>
            let new_entry = #gtk_import::gio::MenuItem::new(None, None);
            let variant = #gtk_import::glib::variant::ToVariant::to_variant(#id);
            new_entry.set_attribute_value("custom", Some(&variant));
            #parent_ident.append_item(&new_entry);
    }
}

impl MenuEntry {
    fn entry_stream(&self, parent_ident: &Ident) -> TokenStream2 {
        let string = &self.string;
        let ty = &self.action_ty;
        if let Some(value) = &self.value {
            quote_spanned! {
                string.span() =>
                    let new_entry = relm4::actions::RelmAction::<#ty>::to_menu_item_with_target_value(#string, &#value);
                    #parent_ident.append_item(&new_entry);
            }
        } else {
            quote_spanned! {
                string.span() =>
                    let new_entry = relm4::actions::RelmAction::<#ty>::to_menu_item(#string);
                    #parent_ident.append_item(&new_entry);
            }
        }
    }
}

impl MenuSection {
    fn section_stream(&self, parent_ident: &Ident) -> TokenStream2 {
        let name = &self.name;
        let gtk_import = crate::gtk_import();
        let mut section_stream = quote! {
            let #name = #gtk_import::gio::Menu::new();
            #parent_ident.append_section(None, &#name);
        };

        for item in &self.items {
            section_stream.extend(item.item_stream(name));
        }

        section_stream
    }
}
