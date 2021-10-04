use proc_macro2::Span;
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
use syn::token::Colon2;

#[derive(Debug)]
pub struct Attrs {
    /// Keeps information about visibility of the widget
    pub visibility: Option<Visibility>,
    /// Allows to track if visibility was ever set
    /// 
    /// You can't set visibility twice.
    /// 
    /// This will fail
    /// ```rust
    /// #[widget(visibility = pub, visibility = pun( in my::path ) ) ]
    /// ```
    /// with error
    ///
    /// ```
    /// ```
    /// 
    visibility_set: bool,

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
            visibility_set: false,

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

        while !input.is_empty() {

            // Attrs::stream_peek(input.cursor());

            if input.peek(Token![pub]) {
                if attrs.visibility_set {
                    return Err(input.error("You can't assign visibility twice"));
                }
                let pub_vis = if input.is_empty() {
                    None
                } else {
                    Some(input.parse()?)
                };
                attrs.visibility = pub_vis;
                attrs.visibility_set = true;
            }
            else if input.peek(Ident) && input.peek2(Token![=]){
                // This allows to report errors at the argument name instead of value
                let visibility_error = Err(input.error("You can't assign visibility twice"));
                let relm4_error = Err(input.error("You can't assign relm4 path twice"));
                let unknown_identifier_error = Err(input.error("Unknown argument. Valid arguments are: `visibility`, `relm4` and `gtk` "));

                let ident: Ident = input.parse()?;
                let _eq: Token![=] = input.parse()?;

                let ident_value: String = ident.to_string();

                if ident_value == "visibility" {
                    if attrs.visibility_set {
                        return visibility_error;
                    }
                    let pub_vis = if input.is_empty() {
                        None
                    } else {
                        Some(input.parse()?)
                    };
                    attrs.visibility = pub_vis;
                    attrs.visibility_set = true;
                } 
                else if ident_value == "relm4" {
                    if attrs.relm4_path_set {
                        return relm4_error;
                    }

                    let path: Path = input.parse()?;
                    
                    attrs.relm4_path = path;
                    attrs.relm4_path_set = true;
                }
                else {
                    return unknown_identifier_error;
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

