use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{GenericArgument, ImplItemMethod};

use crate::component;

pub(super) fn inject_view_code(
    func: Option<ImplItemMethod>,
    view_code: TokenStream2,
    widgets_return_code: TokenStream2,
) -> TokenStream2 {
    if let Some(func) = func {
        match component::inject_view_code::inject_view_code(func, view_code, widgets_return_code) {
            Ok(func) => func.to_token_stream(),
            Err(err) => err.to_compile_error(),
        }
    } else {
        quote! {
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
        }
    }
}
