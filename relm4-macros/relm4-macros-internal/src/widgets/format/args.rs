use quote::ToTokens;
use syn::parse::Parse;

use crate::args::Args;

use super::InlineFormat;

impl<T> InlineFormat for Args<T>
where
    T: ToTokens + Parse,
{
    fn inline_format(&self) -> String {
        let mut output = String::new();
        for arg in &self.inner {
            output.push_str(&arg.to_token_stream().to_string());
            output.push_str(", ");
        }
        output.trim_end_matches(", ").to_owned()
    }
}
