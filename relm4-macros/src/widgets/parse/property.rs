use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token, Error, Ident, Result, Token,
};

use crate::widgets::{
    AssignProperty, Attrs, Property, PropertyName, PropertyType, SignalHandler, Widget, WidgetFunc,
};

impl Parse for Property {
    fn parse(input: ParseStream) -> Result<Self> {
        // Handle `#[attrs]`
        let mut attributes: Option<Attrs> = if input.peek(Token![#]) {
            Some(input.parse()?)
        } else {
            None
        };

        // Parse path, ident or function
        let func: WidgetFunc = input.parse()?;

        // `gtk::Box { ... }`, `data.init_widget() -> gtk::Button { ... }` or `gtk::Box,`
        if input.peek(token::Brace) || input.peek(Token![->]) || input.peek(Token![,]) {
            let ty =
                PropertyType::Widget(Widget::parse_for_container_ext(input, func, attributes)?);

            Ok(Property {
                name: PropertyName::RelmContainerExtAssign,
                ty,
            })
        } else {
            let name = func.into_property_name()?;

            // check for property[a, b, c]: ...
            let mut args = if input.peek(token::Bracket) {
                let paren_input;
                bracketed!(paren_input in input);
                Some(paren_input.parse()?)
            } else {
                None
            };

            // look for event handlers: signal[cloned_data, ...] => move |a, ...| { ... }
            let ty = if input.peek(Token! [=>]) {
                let _arrow: Token![=>] = input.parse()?;
                PropertyType::SignalHandler(SignalHandler::parse_with_args(input, args.take())?)
            }
            // look for widgets
            else if (input.peek(Token![=])
            || input.peek3(Token![=])
            || (input.peek(Token![:]) && input.peek2(Token![mut]) && input.peek3(Ident)))
            // Don't interpret `property: value == other,` as a widget
            && !input.peek3(Token![==])
            {
                if input.peek(Token![=]) {
                    let _token: Token![=] = input.parse()?;
                } else {
                    let _colon: Token![:] = input.parse()?;
                }
                PropertyType::Widget(Widget::parse(input, attributes.take(), args.take())?)
            }
            // look for properties or optional properties (?)
            else if input.peek(Token! [:]) || input.peek(Token! [?]) {
                // look for ? at beginning for optional assign
                PropertyType::Assign(AssignProperty::parse(
                    input,
                    attributes.take(),
                    args.take(),
                )?)
            } else {
                return Err(input.error("Unexpected syntax."));
            };

            // Attributes must have been set to `None` by `take()`
            if let Some(attrs) = attributes {
                if let Some(first_attr) = attrs.inner.first() {
                    return Err(Error::new(
                        first_attr.span(),
                        "No attributes allowed in the following expression.",
                    ));
                }
            }

            // Arguments must have been set to `None` by `take()`
            if let Some(args) = args {
                if let Some(first_arg) = args.inner.first() {
                    return Err(Error::new(
                        first_arg.span(),
                        "No arguments allowed in this expression.",
                    ));
                }
            }

            if !input.is_empty() && !input.peek(Token![,]) {
                Err(input.error("expected `,`. Did you confuse `=` with`:`?"))
            } else {
                Ok(Property { name, ty })
            }
        }
    }
}
