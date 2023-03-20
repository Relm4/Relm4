use std::process::{Command, Stdio};

use quote::ToTokens;

use super::{Format, FormatLine, InlineFormat};

fn call_rustfmt(value: String, pre: &str, post: &str) -> String {
    let input = format!("{pre}{value}{post}");

    let mut rustfmt = Command::new("rustfmt")
        .args(["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    Command::new("echo")
        .arg(&input)
        .stdout(rustfmt.stdin.take().unwrap()) // Converted into a Stdio here
        .spawn()
        .unwrap();

    let std::process::Output { stdout, status, .. } = rustfmt.wait_with_output().unwrap();
    if status.success() {
        let result = stdout
            .iter()
            .skip(pre.as_bytes().len())
            .take(stdout.len() - pre.as_bytes().len() - post.bytes().len() - 1)
            .copied()
            .collect();
        String::from_utf8(result).unwrap()
    } else {
        panic!("Internal error: Failed to format this piece of code:\n`{input}`\nThe target file was NOT changed.");
    }
}

pub(super) fn call_rustfmt_to_lines(
    value: String,
    pre: &str,
    post: &str,
    indent_level: usize,
) -> Vec<FormatLine> {
    let result = call_rustfmt(value, pre, post);
    result
        .lines()
        .map(|line| FormatLine {
            indent_level,
            line: line.to_string(),
        })
        .collect()
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

        call_rustfmt_to_lines(input, "const T: X = ", ";", indent_level)
    }
}

impl Format for syn::ExprClosure {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let input = self.to_token_stream().to_string();

        call_rustfmt_to_lines(input, "const T: X = ", ";", indent_level)
    }
}

impl Format for (&syn::Pat, &Option<(syn::token::If, Box<syn::Expr>)>) {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let (pat, guard) = self;

        let mut input = pat.to_token_stream().to_string();
        if let Some((_, guard)) = guard {
            input = input + "if " + &guard.to_token_stream().to_string();
        }

        call_rustfmt_to_lines(
            input,
            "const T: X = match X {\n    ",
            " => (),\n};",
            indent_level,
        )
    }
}

impl InlineFormat for syn::Type {
    fn inline_format(&self) -> String {
        call_rustfmt(self.to_token_stream().to_string(), "const T: ", " = ();")
    }
}
