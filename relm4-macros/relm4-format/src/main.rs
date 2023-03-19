use std::{fs::ReadDir, path::PathBuf, str::FromStr};

use internal::widgets::{
    format::{Format, FormatLine},
    ViewWidgets,
};

const FILE_ENDINGS: &[&str] = &["rs"];

fn main() {
    let args: Vec<PathBuf> = std::env::args()
        .skip(1)
        .map(|s| PathBuf::from_str(&s).unwrap())
        .collect();

    // Use specified files or otherwise all Rust files in the working directory
    if args.is_empty() {
        let dir = std::fs::read_dir(".").unwrap();
        let mut list = Vec::new();
        walk_dir(&mut list, dir);

        format_files(&list);
    } else {
        format_files(&args);
    }
}

fn walk_dir(list: &mut Vec<PathBuf>, dir: ReadDir) {
    for item in dir {
        let item = item.unwrap();
        let path = item.path();
        if path.is_file()
            && FILE_ENDINGS.contains(&path.extension().unwrap_or_default().to_str().unwrap())
        {
            list.push(path);
            break;
        } else if path.is_dir() {
            let dir = std::fs::read_dir(path).unwrap();
            walk_dir(list, dir);
        }
    }
}

fn format_files(list: &[PathBuf]) {
    for path in list {
        eprintln!("INFO: Formatting {}", path.to_str().unwrap_or_default());
        let contents =
            std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Couldn't open {path:?}"));
        let contents = format_code(&contents);
        std::fs::write(path, contents).unwrap();
    }
}

fn format_code(code: &str) -> String {
    let mut formatted_code = String::new();
    let mut lines = code.lines();

    while let Some(line) = lines.next() {
        if line.trim().starts_with("view!") {
            let initial_ident = line
                .char_indices()
                .take_while(|(_, c)| c.is_whitespace())
                .count();
            let mut macro_code = String::new();
            let mut bracket_level = line.matches('{').count();

            let last_line = loop {
                let line = lines.next().unwrap();
                bracket_level += line.matches('{').count();
                bracket_level = bracket_level.saturating_sub(line.matches('}').count());
                if bracket_level == 0 {
                    break line;
                } else {
                    macro_code.push_str(if line.trim().is_empty() {
                        "#[BLANK]"
                    } else {
                        line
                    });
                    macro_code.push('\n');
                }
            };

            let widgets: ViewWidgets = syn::parse_str(&macro_code).unwrap();
            let lines = widgets.format(initial_ident / 4 + 1);
            let output = concat_format_lines(lines);

            formatted_code.push_str(line);
            formatted_code.push('\n');
            formatted_code.push_str(&output);
            formatted_code.push_str(last_line);
            formatted_code.push('\n');
        } else {
            formatted_code.push_str(line);
            formatted_code.push('\n');
        }
    }

    formatted_code
}

fn concat_format_lines(lines: Vec<FormatLine>) -> String {
    let mut output = String::new();
    for line in lines {
        output.push_str(&" ".repeat(line.indent_level * 4));
        output.push_str(&line.line);
        output.push('\n');
    }
    output
}
