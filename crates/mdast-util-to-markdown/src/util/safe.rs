use std::collections::HashMap;

use regex::Regex;

use crate::state::State;
use crate::types::SafeConfig;
use crate::util::pattern::{compile_pattern, pattern_in_scope};

/// Encode a code point as an HTML character reference.
///
/// Port of JS `lib/util/encode-character-reference.js`.
pub fn encode_character_reference(code: u32) -> String {
    format!("&#x{:X};", code)
}

/// Make a string safe for embedding in markdown constructs.
///
/// Port of JS `lib/util/safe.js`.
///
/// In markdown, almost all punctuation characters can, in certain cases,
/// result in something. Whether they do is highly subjective to where they
/// happen and in what they happen.
///
/// To solve this, `mdast-util-to-markdown` tracks:
/// - Characters before and after something
/// - What "constructs" we are in
///
/// This information is then used to escape or encode special characters.
pub fn safe(state: &State, input: Option<&str>, config: &SafeConfig) -> String {
    let input_str = input.unwrap_or("");
    let value = format!("{}{}{}", config.before, input_str, config.after);

    let mut positions: Vec<usize> = Vec::new();
    let mut infos: HashMap<usize, PositionInfo> = HashMap::new();

    // Find all unsafe positions.
    for pattern in &state.unsafe_patterns {
        if !pattern_in_scope(&state.stack, pattern) {
            continue;
        }

        let expression = compile_pattern(pattern);
        let has_before = pattern.before.is_some() || pattern.at_break;
        let has_after = pattern.after.is_some();

        for mat in expression.find_iter(&value) {
            let before = has_before;
            let after = has_after;

            // Calculate position: if there's a before group, skip past it
            let position = if has_before {
                // The regex has a capture group for the before part.
                // We need to use captures to get the group length.
                // Re-run with captures at this specific match.
                if let Some(caps) = expression.captures(&value[mat.start()..]) {
                    if let Some(group1) = caps.get(1) {
                        mat.start() + group1.len()
                    } else {
                        mat.start()
                    }
                } else {
                    mat.start()
                }
            } else {
                mat.start()
            };

            if let Some(info) = infos.get_mut(&position) {
                if info.before && !before {
                    info.before = false;
                }
                if info.after && !after {
                    info.after = false;
                }
            } else {
                positions.push(position);
                infos.insert(position, PositionInfo { before, after });
            }
        }
    }

    positions.sort();
    positions.dedup();

    let start_offset = config.before.len();
    let end_offset = value.len() - config.after.len();

    let mut result: Vec<String> = Vec::new();
    let mut start = start_offset;

    let ascii_punct = Regex::new(r"[!-/:-@\[-`\{-~]").unwrap();

    for (idx, &position) in positions.iter().enumerate() {
        // Skip positions outside the input range (before/after padding).
        if position < start_offset || position >= end_offset {
            continue;
        }

        // If this character is supposed to be escaped because it has a condition on
        // the next character, and the next character is definitely being escaped,
        // then skip this escape.
        let skip = {
            let pos_info = &infos[&position];
            let next_skip = position + 1 < end_offset
                && idx + 1 < positions.len()
                && positions[idx + 1] == position + 1
                && pos_info.after
                && !infos[&(position + 1)].before
                && !infos[&(position + 1)].after;

            let prev_skip = idx > 0
                && positions[idx - 1] == position - 1
                && pos_info.before
                && !infos[&(position - 1)].before
                && !infos[&(position - 1)].after;

            next_skip || prev_skip
        };

        if skip {
            continue;
        }

        if start != position {
            result.push(escape_backslashes(
                &value[start..position],
                "\\",
            ));
        }

        start = position;

        let ch = &value[position..position + value[position..].chars().next().unwrap().len_utf8()];

        if ascii_punct.is_match(ch) && !config.encode.contains(&ch.chars().next().unwrap()) {
            // Character escape with backslash.
            result.push("\\".to_string());
        } else {
            // Character reference.
            result.push(encode_character_reference(
                ch.chars().next().unwrap() as u32,
            ));
            start += ch.len();
        }
    }

    result.push(escape_backslashes(
        &value[start..end_offset],
        &config.after,
    ));

    result.join("")
}

/// Info about why a position was marked unsafe.
#[derive(Debug, Clone)]
struct PositionInfo {
    before: bool,
    after: bool,
}

/// Check if a character is ASCII punctuation (matches JS regex `[!-/:-@[-`{-~]`).
fn is_ascii_punct(ch: char) -> bool {
    matches!(ch, '!'..='/' | ':'..='@' | '['..='`' | '{'..='~')
}

/// Escape existing backslashes that precede ASCII punctuation.
///
/// Port of the JS `escapeBackslashes` inner function in `safe.js`.
/// The JS version uses `\\(?=[!-/:-@[-`{-~])` regex with lookahead.
/// Since the `regex` crate doesn't support lookahead, we manually scan.
fn escape_backslashes(value: &str, after: &str) -> String {
    let whole = format!("{}{}", value, after);
    let whole_bytes = whole.as_bytes();

    // Find positions of backslashes followed by ASCII punctuation in the whole string
    let mut positions: Vec<usize> = Vec::new();
    for i in 0..whole_bytes.len().saturating_sub(1) {
        if whole_bytes[i] == b'\\' && is_ascii_punct(whole_bytes[i + 1] as char) {
            positions.push(i);
        }
    }

    let mut results: Vec<&str> = Vec::new();
    let mut start = 0;

    for &pos in &positions {
        // Only process positions within the value (not in the after part)
        if pos < value.len() {
            if start != pos {
                results.push(&value[start..pos]);
            }
            results.push("\\");
            start = pos;
        }
    }

    results.push(&value[start..]);

    results.join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_character_reference() {
        assert_eq!(encode_character_reference(0x26), "&#x26;");
        assert_eq!(encode_character_reference(0x3C), "&#x3C;");
    }

    #[test]
    fn test_escape_backslashes() {
        // Backslash before punctuation should be double-escaped
        assert_eq!(escape_backslashes("a\\*b", ""), "a\\\\*b");
        // Backslash not before punctuation should be left alone
        assert_eq!(escape_backslashes("a\\b", ""), "a\\b");
    }
}
