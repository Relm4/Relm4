use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Ident, LitStr};

use super::{Menu, MenuElement, MenuEntry, MenuItem, MenuSection, Menus, SubMenu};

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

        // Create new menu
        let mut menu_stream = quote_spanned! {
            name.span() =>
                let #name = #gtk_import ::gio::Menu::new();
        };

        // Add items
        for item in &self.items {
            menu_stream.extend(item.item_stream(name));
        }

        menu_stream
    }
}

impl MenuElement {
    fn item_stream(&self, parent_ident: &Ident) -> TokenStream2 {
        let mut item_stream = TokenStream2::new();

        item_stream.extend(match self {
            Self::Item(entry) => entry.item_stream(parent_ident),
            Self::Section(section) => section.section_stream(parent_ident),
            Self::Custom(id) => custom_stream(parent_ident, id),
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

impl MenuItem {
    fn item_stream(&self, parent_ident: &Ident) -> TokenStream2 {
        match self {
            Self::Entry(entry) => entry.entry_stream(parent_ident),
            Self::SubMenu(sub_menu) => sub_menu.submenu_stream(parent_ident),
        }
    }
}

impl SubMenu {
    fn submenu_stream(&self, parent_ident: &Ident) -> TokenStream2 {
        let name = Ident::new(&format!("_{parent_ident}"), Span2::mixed_site());
        let gtk_import = crate::gtk_import();
        let expr = &self.expr;

        // Create new sub-menu
        let mut item_stream = quote_spanned! {
            expr.span() =>
                let #name = #gtk_import ::gio::Menu::new();
                #parent_ident.append_submenu(Some(#expr), &#name);
        };

        // Add items
        for item in &self.items {
            item_stream.extend(item.item_stream(&name));
        }

        // Wrap the generated code in a new scope to avoid side-effects
        quote! {
            {
                #item_stream
            }
        }
    }
}

impl MenuEntry {
    fn entry_stream(&self, parent_ident: &Ident) -> TokenStream2 {
        let expr = &self.expr;
        let ty = &self.action_ty;

        if let Some(value) = &self.value {
            quote_spanned! {
                expr.span() =>
                    let new_entry = relm4::actions::RelmAction::<#ty>::to_menu_item_with_target_value(#expr, &#value);
                    #parent_ident.append_item(&new_entry);
            }
        } else {
            quote_spanned! {
                expr.span() =>
                    let new_entry = relm4::actions::RelmAction::<#ty>::to_menu_item(#expr);
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
