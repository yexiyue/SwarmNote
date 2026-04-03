use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::types::{ConstructName, UnsafePattern};

/// Cache for compiled patterns (keyed by a string representation of the pattern).
///
/// In the JS version, patterns are mutated to cache the compiled regex on the
/// pattern object itself (`pattern._compiled`). In Rust we use a separate cache.
static REGEX_METACHAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"[|\\{}\(\)\[\]\^$+*?.\-]").unwrap());

/// Compile an unsafe pattern to a regex string and create a `Regex`.
///
/// Port of JS `lib/util/compile-pattern.js`.
///
/// The compiled regex matches the unsafe character in context:
/// - If `at_break` is set, matches `[\r\n][\t ]*` before the character
/// - If `before` is set, matches the before pattern as a lookbehind group
/// - The character itself (escaped if it's a regex metacharacter)
/// - If `after` is set, matches the after pattern after the character
///
/// The regex is global (finds all matches).
pub fn compile_pattern(pattern: &UnsafePattern) -> Regex {
    let before = {
        let at_break_part = if pattern.at_break {
            "[\\r\\n][\\t ]*"
        } else {
            ""
        };
        let before_part = match &pattern.before {
            Some(b) => format!("(?:{})", b),
            None => String::new(),
        };
        format!("{}{}", at_break_part, before_part)
    };

    let char_str = {
        let c = pattern.character.to_string();
        if REGEX_METACHAR.is_match(&c) {
            format!("\\{}", c)
        } else {
            c
        }
    };

    let after_part = match &pattern.after {
        Some(a) => format!("(?:{})", a),
        None => String::new(),
    };

    let regex_str = if before.is_empty() {
        format!("{}{}", char_str, after_part)
    } else {
        format!("({}){}{}", before, char_str, after_part)
    };

    Regex::new(&regex_str).unwrap()
}

/// Thread-safe compiled pattern cache.
///
/// Uses a string key derived from the pattern fields.
pub struct PatternCache {
    cache: HashMap<String, Regex>,
}

impl PatternCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Get or compile a regex for the given pattern.
    pub fn get_or_compile(&mut self, pattern: &UnsafePattern) -> &Regex {
        let key = pattern_cache_key(pattern);
        self.cache
            .entry(key)
            .or_insert_with(|| compile_pattern(pattern))
    }
}

impl Default for PatternCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a cache key for an unsafe pattern.
fn pattern_cache_key(pattern: &UnsafePattern) -> String {
    format!(
        "{}|{}|{}|{}",
        pattern.character,
        pattern.before.as_deref().unwrap_or(""),
        pattern.after.as_deref().unwrap_or(""),
        pattern.at_break,
    )
}

/// Check whether an unsafe pattern is in scope given the current construct stack.
///
/// Port of JS `lib/util/pattern-in-scope.js`.
///
/// A pattern is in scope when:
/// 1. Its `in_construct` list matches (any construct in the stack is in the list,
///    or the list is empty which means "always active")
/// 2. Its `not_in_construct` list does NOT match (none of the constructs in the
///    stack are in the not-in list)
pub fn pattern_in_scope(stack: &[ConstructName], pattern: &UnsafePattern) -> bool {
    list_in_scope(stack, &pattern.in_construct, true)
        && !list_in_scope(stack, &pattern.not_in_construct, false)
}

/// Check whether any construct in the stack matches any construct in the list.
///
/// If the list is empty, returns `none` (the default for "no constraint").
fn list_in_scope(stack: &[ConstructName], list: &[ConstructName], none: bool) -> bool {
    if list.is_empty() {
        return none;
    }

    for item in list {
        if stack.contains(item) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_basic_pattern() {
        let pattern = UnsafePattern {
            character: '*',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![],
            not_in_construct: vec![],
        };
        let re = compile_pattern(&pattern);
        assert!(re.is_match("hello * world"));
    }

    #[test]
    fn test_compile_pattern_with_before_after() {
        let pattern = UnsafePattern {
            character: '.',
            before: Some("\\d+".to_string()),
            after: Some("(?:[ \\t\\r\\n]|$)".to_string()),
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        };
        let re = compile_pattern(&pattern);
        // Should match "1." at start of line (after break)
        assert!(re.is_match("\n1. item"));
    }

    #[test]
    fn test_pattern_in_scope_empty_lists() {
        let stack = vec![ConstructName::Phrasing];
        let pattern = UnsafePattern {
            character: '*',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![],
            not_in_construct: vec![],
        };
        // Empty in_construct -> always active; empty not_in_construct -> not excluded
        assert!(pattern_in_scope(&stack, &pattern));
    }

    #[test]
    fn test_pattern_in_scope_matching() {
        let stack = vec![ConstructName::Phrasing];
        let pattern = UnsafePattern {
            character: '*',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: vec![],
        };
        assert!(pattern_in_scope(&stack, &pattern));
    }

    #[test]
    fn test_pattern_in_scope_not_matching() {
        let stack = vec![ConstructName::HeadingAtx];
        let pattern = UnsafePattern {
            character: '*',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: vec![],
        };
        assert!(!pattern_in_scope(&stack, &pattern));
    }

    #[test]
    fn test_pattern_not_in_scope_excluded() {
        let stack = vec![ConstructName::Phrasing, ConstructName::Autolink];
        let pattern = UnsafePattern {
            character: '*',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: vec![ConstructName::Autolink],
        };
        assert!(!pattern_in_scope(&stack, &pattern));
    }
}
