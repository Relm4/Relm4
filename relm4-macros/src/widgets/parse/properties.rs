use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Result, Token,
};

use crate::widgets::{Properties, Property};

impl Parse for Properties {
    fn parse(input: ParseStream) -> Result<Self> {
        let props: Punctuated<Property, Token![,]> = input.parse_terminated(Property::parse)?;
        let properties = props.into_pairs().map(|pair| pair.into_value()).collect();
        Ok(Properties { properties })
    }
}
