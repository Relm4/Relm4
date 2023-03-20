use quote::ToTokens;

use crate::widgets::ReturnedWidget;

use super::{Format, FormatLine};

impl Format for ReturnedWidget {
    fn format(&self, indent_level: usize) -> Vec<super::FormatLine> {
        let ReturnedWidget {
            name_assigned_by_user,
            name,
            ty,
            properties,
            is_optional,
        } = self;

        let mut line = "} ->".to_owned();

        if *name_assigned_by_user {
            line = line + " " + &name.to_string();
        }

        if let Some(ty) = ty {
            line = line + ": " + &ty.into_token_stream().to_string();
        }

        if *is_optional {
            line += "?";
        }

        line += " {";

        let mut output = vec![FormatLine { indent_level, line }];

        for property in &properties.properties {
            output.extend(property.format(indent_level + 1));
        }

        output
    }
}
