use regex::Regex;

/// Type for the map function used by `indent_lines` (function pointer variant).
///
/// Arguments: (line_content, line_number_0_indexed, is_blank) -> padded line.
pub type IndentMap = fn(&str, usize, bool) -> String;

/// Pad serialized markdown by applying a map function to each line.
///
/// Port of JS `lib/util/indent-lines.js`.
///
/// Splits `value` by line endings, applies `map` to each line fragment,
/// and reassembles with the original line endings preserved.
pub fn indent_lines<F>(value: &str, map: F) -> String
where
    F: Fn(&str, usize, bool) -> String,
{
    let eol = Regex::new(r"\r?\n|\r").unwrap();
    let mut result: Vec<String> = Vec::new();
    let mut start = 0;
    let mut line = 0;

    for mat in eol.find_iter(value) {
        let fragment = &value[start..mat.start()];
        one(&mut result, fragment, line, &map);
        result.push(mat.as_str().to_string());
        start = mat.end();
        line += 1;
    }

    // Handle the last fragment after the last line ending (or the whole string if no line endings).
    let fragment = &value[start..];
    one(&mut result, fragment, line, &map);

    result.join("")
}

/// Process a single line fragment through the map function.
fn one<F>(result: &mut Vec<String>, value: &str, line: usize, map: &F)
where
    F: Fn(&str, usize, bool) -> String,
{
    result.push(map(value, line, value.is_empty()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_lines_basic() {
        fn add_prefix(value: &str, _line: usize, blank: bool) -> String {
            if blank {
                String::new()
            } else {
                format!("> {}", value)
            }
        }

        let result = indent_lines("hello\nworld", add_prefix);
        assert_eq!(result, "> hello\n> world");
    }

    #[test]
    fn test_indent_lines_with_blank() {
        fn add_prefix(value: &str, _line: usize, blank: bool) -> String {
            if blank {
                String::new()
            } else {
                format!("  {}", value)
            }
        }

        let result = indent_lines("a\n\nb", add_prefix);
        assert_eq!(result, "  a\n\n  b");
    }
}
