use proc_macro2::Span;
use syn::Error;
use syn::Ident;
use syn::Path;
use syn::PathArguments;
use syn::PathSegment;
use syn::Result;
use syn::Token;
use syn::Visibility;
// use syn::buffer::Cursor;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Colon2;

#[derive(Debug)]
enum AttributeType {
    None,
    Named,
    Unnamed{
        span: Span
    },
}

#[derive(Debug)]
pub struct Attrs {
    /// Keeps information about visibility of the widget
    pub visibility: Option<Visibility>,

    /// Path to relm4
    /// 
    /// Defaults to `::relm4`
    pub relm4_path: Path,
    /// Allows to track if relm4 path was already set
    /// 
    /// You can't set relm4 path twice
    /// 
    /// ```rust
    /// #[widget(relm4 = ::my::path, relm4 = ::my::other::path ) ]
    /// ```
    /// is illegal
    relm4_path_set: bool,
}

impl Attrs {
    fn new() -> Self {
        let relm4_path_segment = PathSegment {
            ident: Ident::new("relm4", Span::call_site()),
            arguments: PathArguments::None,
        };
        
        let mut relm4_segments: Punctuated<PathSegment, Colon2> = Punctuated::new();
        relm4_segments.push(relm4_path_segment);

        let relm4_path: Path = Path {
            leading_colon: Some(Token![::](Span::call_site())),
            segments: relm4_segments,
        };

        Attrs {
            visibility: None,

            relm4_path,
            relm4_path_set: false,
        }
    }

    // fn stream_peek(c: Cursor) {
    //     eprintln!("\tpeek: ");
    //     Attrs::stream_peek_print(c);
    // }

    // fn stream_peek_print(c: Cursor) {
    //     if !c.eof() {
    //         //group it is not
    //         if let Some( (ident, cursor) ) = c.ident() {
    //             eprintln!("\t\t ident: {}", ident);
    //             return Attrs::stream_peek_print(cursor);
    //         }

    //         if let Some( (punct, cursor) ) = c.punct() {
    //             eprintln!("\t\t punct: {}", punct);
    //             return Attrs::stream_peek_print(cursor);
    //         }

    //         if let Some( (literal, cursor) ) = c.literal() {
    //             eprintln!("\t\t literal: {}", literal);
    //             return Attrs::stream_peek_print(cursor);
    //         }

    //         if let Some( (lifetime, cursor) ) = c.lifetime() {
    //             eprintln!("\t\t lifetime: {}", lifetime);
    //             return Attrs::stream_peek_print(cursor);
    //         }

    //         if let Some( (tree, cursor) ) = c.token_tree() {
    //             eprintln!("\t\t tree: {}", tree);
    //             Attrs::stream_peek_print(cursor);
    //         }
    //     }
    // }
}

impl Parse for Attrs {
    /// Rules for parsing attributes
    /// 
    /// 1. It's fine if visibility is used unnamed so `#[widget(pub)]` must be valid
    /// 2. Widget visibility might be named `#[widget(visibility = pub)]`
    ///
    fn parse(input: ParseStream) -> Result<Self> {
        // eprintln!("Input:");
        // eprintln!("\tis empty: {}", input.is_empty());

        let mut attrs = Attrs::new();
        let mut attrs_type = AttributeType::None;

        let mixed_use_error_message = 
            "You can't mix named and unnamed arguments while using `relm4_macros::widget`. \n\
            \n\
            You can use one of\n\
            \n\
            1. `#[relm4_macros::widget()]` to define widget with private visibility\n\
            2. `#[relm4_macros::widget(pub)]` to define widget with public visibility\n\
            3. `#[relm4_macros::widget( visibility = pub )]` to define widget with public visibility and potentially other arguments\n\
            \n\
            Please use `visibility = pub` to fix this error";

        while !input.is_empty() {

            // Attrs::stream_peek(input.cursor());

            if input.peek(Token![pub]) {
                if matches!(attrs_type, AttributeType::Named) {
                    return Err(input.error(mixed_use_error_message));
                }
                if attrs.visibility.is_some() {
                    return Err(input.error("You can't assign visibility twice"));
                }
                let pub_vis: Visibility = input.parse()?;

                attrs_type = AttributeType::Unnamed{
                    span: pub_vis.span(),
                };
                attrs.visibility = Some(pub_vis);
            }
            else if input.peek(Ident) && input.peek2(Token![=]){
                let ident: Ident = input.parse()?;
                let eq: Token![=] = input.parse()?;

                if ident == "visibility" {
                    let pub_vis: Visibility = input.parse()?;
                    let error_span = ident.span().join(eq.span).unwrap()
                                        .join(pub_vis.span()).unwrap();

                    if let AttributeType::Unnamed{ span } = attrs_type {
                        return Err(Error::new(span, mixed_use_error_message));
                    }
                    if attrs.visibility.is_some() {
                        return Err(Error::new(error_span, "You can't assign visibility twice"));
                    }
                    
                    attrs.visibility = Some(pub_vis);
                    attrs_type = AttributeType::Named;
                }
                else if ident == "relm4" {
                    let path: Path = input.parse()?;
                    
                    if let AttributeType::Unnamed{ span } = attrs_type {
                        return Err(Error::new(span, mixed_use_error_message));
                    }
                    if attrs.relm4_path_set {
                        let error_span = ident.span().join(eq.span).unwrap()
                                            .join(path.span()).unwrap();
                        return Err(Error::new(error_span, "You can't assign relm4 path twice"));
                    }

                    attrs.relm4_path = path;
                    attrs.relm4_path_set = true;
                    attrs_type = AttributeType::Named;
                }
                else {
                    return Err(input.error("Unknown argument. Valid arguments are: `visibility` or `relm4`"));
                }
            }

            if input.peek(Token![,]) {
                let _comma: Token![,] = input.parse()?;
            } else {
                break;
            }
        }

        Ok(attrs)
    }

    
}

