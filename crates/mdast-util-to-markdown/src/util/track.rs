use crate::types::TrackFields;

/// Position tracker for generated markdown output.
///
/// Port of JS `lib/util/track.js`.
///
/// Tracks the current line, column, and line shift (indent) as markdown
/// content is generated. This info can be used for source maps, line wrapping, etc.
pub struct Tracker {
    line: usize,
    column: usize,
    line_shift: usize,
}

impl Tracker {
    /// Create a new tracker from tracking fields.
    pub fn new(fields: &TrackFields) -> Self {
        Self {
            line: fields.line,
            column: fields.column,
            line_shift: fields.line_shift,
        }
    }

    /// Get the current tracked position info.
    pub fn current(&self) -> TrackFields {
        TrackFields {
            line: self.line,
            column: self.column,
            line_shift: self.line_shift,
        }
    }

    /// Define a relative increased line shift (the typical indent for lines).
    pub fn shift(&mut self, value: usize) {
        self.line_shift += value;
    }

    /// Move past some generated markdown.
    ///
    /// Updates line/column tracking based on the content of `value`.
    /// Returns the input value unchanged (for chaining).
    pub fn r#move(&mut self, value: &str) -> String {
        let chunks: Vec<&str> = split_lines(value);
        let tail = chunks.last().unwrap_or(&"");
        self.line += chunks.len() - 1;
        self.column = if chunks.len() == 1 {
            self.column + tail.len()
        } else {
            1 + tail.len() + self.line_shift
        };
        value.to_string()
    }
}

/// Split a string by line endings (same as JS `value.split(/\r?\n|\r/g)`).
fn split_lines(value: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let bytes = value.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'\r' {
            result.push(&value[start..i]);
            if i + 1 < len && bytes[i + 1] == b'\n' {
                i += 2;
            } else {
                i += 1;
            }
            start = i;
        } else if bytes[i] == b'\n' {
            result.push(&value[start..i]);
            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }

    result.push(&value[start..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_basic() {
        let fields = TrackFields {
            line: 1,
            column: 1,
            line_shift: 0,
        };
        let mut tracker = Tracker::new(&fields);

        tracker.r#move("hello");
        let current = tracker.current();
        assert_eq!(current.line, 1);
        assert_eq!(current.column, 6);

        tracker.r#move("\nworld");
        let current = tracker.current();
        assert_eq!(current.line, 2);
        assert_eq!(current.column, 6);
    }

    #[test]
    fn test_tracker_with_shift() {
        let fields = TrackFields {
            line: 1,
            column: 1,
            line_shift: 2,
        };
        let mut tracker = Tracker::new(&fields);

        tracker.r#move("hello\nworld");
        let current = tracker.current();
        assert_eq!(current.line, 2);
        // 1 + "world".len() + line_shift(2) = 1 + 5 + 2 = 8
        assert_eq!(current.column, 8);
    }

    #[test]
    fn test_split_lines() {
        assert_eq!(split_lines("a\nb\nc"), vec!["a", "b", "c"]);
        assert_eq!(split_lines("a\r\nb"), vec!["a", "b"]);
        assert_eq!(split_lines("a\rb"), vec!["a", "b"]);
        assert_eq!(split_lines("abc"), vec!["abc"]);
        assert_eq!(split_lines(""), vec![""]);
    }
}
