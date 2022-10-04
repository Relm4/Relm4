use proc_macro2::TokenStream as TokenStream2;

use syn::{parse_quote, ImplItemMethod, Result};

use crate::component;

pub(super) fn inject_view_code(
    func: Option<ImplItemMethod>,
    view_code: &TokenStream2,
    widgets_return_code: &TokenStream2,
) -> Result<ImplItemMethod> {
    if let Some(func) = func {
        Ok(component::inject_view_code::inject_view_code(
            func,
            view_code,
            widgets_return_code,
        )?)
    } else {
        Ok(parse_quote! {
            fn init_widgets(
                &mut self,
                index: &relm4::factory::DynamicIndex,
                root: &Self::Root,
                returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
                sender: relm4::factory::FactoryComponentSender<Self>,
            ) -> Self::Widgets {
                #view_code
                #widgets_return_code
            }
        })
    }
}
