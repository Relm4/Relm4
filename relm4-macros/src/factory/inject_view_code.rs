use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{ImplItemMethod, Path};

use crate::component;

pub(super) fn inject_view_code(
    func: Option<ImplItemMethod>,
    view_code: TokenStream2,
    widgets_return_code: TokenStream2,
    container_widget: TokenStream2,
    relm4_path: &Path,
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
                index: &#relm4_path::factory::DynamicIndex,
                root: &Self::Root,
                returned_widget: &<#container_widget as #relm4_path::factory::FactoryView>::ReturnedWidget,
                input: &Sender<Self::Input>,
                output: &Sender<Self::Output>,
            ) -> Self::Widgets {
                #view_code
                #widgets_return_code
            }
        }
    }
}
