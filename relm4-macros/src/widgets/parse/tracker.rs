use syn::{
    parse::{Parse, ParseStream},
    Result, Token,
};

use crate::widgets::Tracker;

impl Parse for Tracker {
    fn parse(input: ParseStream) -> Result<Self> {
        let bool_fn = input.parse()?;

        let mut update_fns = Vec::new();
        while !input.is_empty() {
            let _comma: Token![,] = input.parse()?;
            // allow comma at the end of the macro
            if !input.is_empty() {
                update_fns.push(input.parse()?);
            }
        }

        Ok(Tracker {
            bool_fn,
            update_fns,
        })
    }
}
