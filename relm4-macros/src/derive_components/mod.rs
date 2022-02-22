use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parenthesized, parse::Parse, parse_macro_input, spanned::Spanned, Data, DeriveInput, Error,
    Fields, GenericArgument, GenericParam, Ident, Path, PathArguments, Result, Token, Type,
    TypePath,
};

#[derive(Debug)]
enum ComponentsAttr {
    RelmPath(Path),
    ModelPath(Path),
}

impl Parse for ComponentsAttr {
    fn parse(paren_input: syn::parse::ParseStream) -> Result<Self> {
        let input;
        parenthesized!(input in paren_input);

        if input.peek2(Token![=]) {
            let ident: Ident = input.parse()?;
            let _eq: Token![=] = input.parse()?;
            match ident.to_string().as_str() {
                "relm4" => Ok(ComponentsAttr::RelmPath(input.parse()?)),
                "parent_model" => Ok(ComponentsAttr::ModelPath(input.parse()?)),
                _ => Err(input.error("Expected either 'relm4' or 'parent_model'")),
            }
        } else {
            Ok(ComponentsAttr::ModelPath(input.parse()?))
        }
    }
}

pub(super) fn generate_stream(input: &DeriveInput) -> Result<TokenStream2> {
    let ident = &input.ident;

    let data = if let Data::Struct(data) = &input.data {
        data
    } else {
        return Err(Error::new(input.span(), "Expected a struct."));
    };

    let fields = if let Fields::Named(fields) = &data.fields {
        fields
    } else {
        return Err(Error::new(input.span(), "Expected a named struct fields."));
    };

    let mut relm4_path = quote! { relm4 };
    let mut model = None;
    for attr in &input.attrs {
        let ident = &attr.path.segments.last().unwrap().ident;
        if ident == "components" {
            let attr: ComponentsAttr = parse_macro_input::parse(attr.tokens.clone().into())?;
            match attr {
                ComponentsAttr::RelmPath(path) => relm4_path = quote! { #path },
                ComponentsAttr::ModelPath(path) => model = Some(path),
            }
        }
    }

    // Remove default type parameters (like <Type=DefaultType>).
    let mut generics = input.generics.clone();
    for param in generics.params.iter_mut() {
        if let GenericParam::Type(ty) = param {
            ty.eq_token = None;
            ty.default = None;
        }
    }

    let mut init_stream = TokenStream2::new();
    let mut connect_parent_stream = TokenStream2::new();

    for field in fields.named.iter() {
        if let Type::Path(type_path) = &field.ty {
            let last_segment = type_path.path.segments.last().unwrap();
            let ident = &field.ident;

            // Remove path arguments
            let mut path = type_path.clone();
            path.path
                .segments
                .iter_mut()
                .for_each(|p| p.arguments = PathArguments::None);

            match last_segment.ident.to_string().as_str() {
                "RelmComponent" => {
                    init_stream.extend(quote! {
                        #ident: #path ::new(model, sender.clone()),
                    });
                    connect_parent_stream.extend(quote! {
                        self. #ident.connect_parent(parent_widgets);
                    });
                }
                "RelmWorker" => {
                    init_stream.extend(quote! {
                        #ident: #path ::with_new_thread(model, sender.clone()),
                    });
                }
                "AsyncRelmWorker" => {
                    init_stream.extend(quote! {
                        #ident: #path ::with_new_tokio_rt(model, sender.clone()),
                    });
                }
                "RelmMsgHandler" => {
                    init_stream.extend(quote! {
                        #ident: #path ::new(model, sender.clone()),
                    });
                }
                _ => {
                    return Err(Error::new(path.span(), "Expected a Relm4 component type here (such as RelmComponent or RelmWorker). For other types please manually implement the trait."));
                }
            }
            if model.is_none() {
                model = Some(last_path_generics(type_path)?);
            }
        }
    }

    let model =
        model.ok_or_else(|| Error::new(ident.span(), "Expected attribute for model parameter"))?;

    let stream = quote! {
        impl #generics #relm4_path ::Components< #model > for #ident {
            fn init_components(model: & #model, sender: Sender<<#model as #relm4_path::Model>::Msg>) -> Self {
                Self {
                    #init_stream
                }
            }

            fn connect_parent(&mut self, parent_widgets: & <#model as #relm4_path::Model>::Widgets) {
                #connect_parent_stream
            }
        }
    };

    Ok(stream)
}

fn last_path_generics(path: &TypePath) -> Result<Path> {
    if let PathArguments::AngleBracketed(generics) = &path.path.segments.last().unwrap().arguments {
        let last_arg = generics.args.last().unwrap();
        if let GenericArgument::Type(Type::Path(path)) = last_arg {
            Ok(path.path.clone())
        } else {
            Err(Error::new(
                last_arg.span(),
                "Expected the last generic argument to be a type path.",
            ))
        }
    } else {
        Err(Error::new(
            path.span(),
            "Expected to find angle bracketed generics.",
        ))
    }
}
