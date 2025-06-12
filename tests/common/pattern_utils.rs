#![allow(dead_code)]

use bc_envelope::prelude::*;

// Format each path element on its own line, each line successively indented by
// 4 spaces.
pub fn format_path(path: &Path) -> String {
    let mut lines = Vec::new();
    for (i, element) in path.iter().enumerate() {
        let indent = " ".repeat(i * 4);
        lines.push(format!(
            "{}{} {}",
            indent,
            element.short_id(DigestDisplayFormat::Short),
            element.format_flat()
        ));
    }
    lines.join("\n")
}

pub fn format_paths(paths: &[Path]) -> String {
    paths.iter().map(format_path).collect::<Vec<_>>().join("\n")
}
