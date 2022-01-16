use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Result, Token,
};

use super::{Properties, Property};

mod property;
mod property_name;
mod returned_widget;
mod tracker;
mod widget;
mod widget_func;

impl Parse for Properties {
    fn parse(input: ParseStream) -> Result<Self> {
        let props: Punctuated<Property, Token![,]> = input.parse_terminated(Property::parse)?;
        let properties = props.into_pairs().map(|pair| pair.into_value()).collect();
        Ok(Properties { properties })
    }
}
