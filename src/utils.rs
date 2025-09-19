use std::ops::Range;

pub type Span = Range<usize>;
pub type Spanned<T> = (T, Span);

//#[derive(Debug, PartialEq)]
//pub struct Spanned<T>(T, Span);

/// Preprocess source to handle empty lines in indented blocks and ensure newline ending
pub fn preprocess_src(src: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let mut preprocessed_lines = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() && i > 0 && i < lines.len() - 1 {
            // This is an empty line between other lines
            let prev_line = lines[i - 1];
            let next_line = lines[i + 1];

            // Get indentation of previous and next lines
            let prev_indent = prev_line.len() - prev_line.trim_start().len();
            let next_indent = next_line.len() - next_line.trim_start().len();

            // Check if both previous and next lines are indented and have content
            let prev_indented = prev_indent > 0 && !prev_line.trim().is_empty();
            let next_indented = next_indent > 0 && !next_line.trim().is_empty();

            if prev_indented && next_indented {
                // Insert a dummy comment with the same indentation as the next line
                let indent_spaces = " ".repeat(next_indent);
                preprocessed_lines.push(format!("{}# dummy", indent_spaces));
            } else {
                preprocessed_lines.push(line.to_string());
            }
        } else {
            preprocessed_lines.push(line.to_string());
        }
    }

    let mut result = preprocessed_lines.join("\n");

    // Ensure the string ends with a newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    result
}
