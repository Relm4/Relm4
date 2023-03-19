use std::process::{Command, Stdio};

use quote::ToTokens;

use super::{Format, FormatLine, InlineFormat};

pub(super) fn call_rustfmt(value: String, pre: &str, post: &str) -> String {
    let input = format!("{pre}{value}{post}");

    let mut rustfmt = Command::new("rustfmt")
        .args(["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    Command::new("echo")
        .arg(input)
        .stdout(rustfmt.stdin.take().unwrap()) // Converted into a Stdio here
        .spawn()
        .unwrap();

    let result = rustfmt.wait_with_output().unwrap().stdout;
    let result = result
        .iter()
        .skip(pre.as_bytes().len())
        .take(result.len() - pre.as_bytes().len() - post.bytes().len() - 1)
        .copied()
        .collect();
    String::from_utf8(result).unwrap()
}

impl InlineFormat for syn::Path {
    fn inline_format(&self) -> String {
        let input = self.to_token_stream().to_string();

        call_rustfmt(input, "type T = ", ";")
    }
}

impl Format for syn::Expr {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let input = self.to_token_stream().to_string();

        let result = call_rustfmt(input, "const T: X = ", ";");
        result
            .lines()
            .map(|line| FormatLine {
                indent_level,
                line: line.to_string(),
            })
            .collect()
    }
}

impl Format for syn::ExprClosure {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let input = self.to_token_stream().to_string();

        let result = call_rustfmt(input, "const T: X = ", ";");
        result
            .lines()
            .map(|line| FormatLine {
                indent_level,
                line: line.to_string(),
            })
            .collect()
    }
}
