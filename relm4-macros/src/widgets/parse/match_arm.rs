use syn::parse::ParseStream;
use syn::token::And;
use syn::{Token, token};

use crate::widgets::parse::if_branch::args_from_index;
use crate::widgets::{MatchArm, ParseError, Widget, parse_util};

impl MatchArm {
    pub(super) fn parse(input: ParseStream<'_>, index: usize) -> Result<Self, ParseError> {
        let pattern = syn::Pat::parse_multi_with_leading_vert(input)?;
        let guard = if input.peek(token::FatArrow) {
            None
        } else {
            Some((input.parse()?, input.parse()?))
        };

        let arrow = input.parse()?;

        let braced;
        let inner_tokens = if input.peek(token::Brace) {
            braced = parse_util::braces(input)?;
            &braced
        } else {
            input
        };

        let attributes = inner_tokens.parse().ok();
        let args = args_from_index(index, input.span());

        let ref_span = input.span();
        let mut widget = Widget::parse(inner_tokens, attributes, Some(args))?;
        widget.ref_token = Some(And { spans: [ref_span] });

        // Parse trailing commas
        if input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
        }

        Ok(Self {
            pattern,
            guard,
            arrow,
            widget,
        })
    }
}
