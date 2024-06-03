use std::{env, fs::OpenOptions, io::Write};

const CLASSES: [&str; 40] = [
    "accent",
    "activatable",
    "background",
    "body",
    "boxed-list",
    "caption",
    "caption-heading",
    "card",
    "circular",
    "compact",
    "destructive-action",
    "devel",
    "dim-label",
    "error",
    "flat",
    "frame",
    "heading",
    "icon-dropshadow",
    "inline",
    "linked",
    "lowres-icon",
    "menu",
    "monospace",
    "navigation-sidebar",
    "numeric",
    "opaque",
    "osd",
    "pill",
    "raised",
    "selection-mode",
    "spacer",
    "success",
    "suggested-action",
    "title-1",
    "title-2",
    "title-3",
    "title-4",
    "toolbar",
    "view",
    "warning",
];

const DEPCRECATED_CLASSES: [&str; 4] = ["app-notification", "content", "large-title", "sidebar"];

const COLORS: [&str; 9] = [
    "blue", "green", "yellow", "orange", "red", "purple", "brown", "light", "dark",
];

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let classes_path = format!("{out_dir}/classes.rs");
    let colors_path = format!("{out_dir}/colors.rs");

    let mut classes_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(classes_path)
        .unwrap();
    let mut colors_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(colors_path)
        .unwrap();

    let classes = CLASSES.into_iter().chain(DEPCRECATED_CLASSES);
    for class in classes {
        let var_name = class.to_uppercase().replace('-', "_");

        if DEPCRECATED_CLASSES.contains(&class) {
            writeln!(
                classes_file,
                r#"#[deprecated(note = "Adwaita has deprecated this CSS class")]"#
            )
            .unwrap();
        }
        writeln!(classes_file, "/// The `{class}` CSS class.").unwrap();
        writeln!(classes_file, r#"pub const {var_name}: &str = "{class}";"#).unwrap();
    }

    for color in COLORS {
        for num in 1..=5 {
            let var_name = format!("{}_{num}", color.to_uppercase());

            writeln!(colors_file, "/// The `@{color}_{num}` palette color.").unwrap();
            writeln!(
                colors_file,
                r#"pub const {var_name}: &str = "@{color}_{num}";"#
            )
            .unwrap();
        }
    }
}
