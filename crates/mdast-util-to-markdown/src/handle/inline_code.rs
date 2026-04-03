use markdown::mdast::Node;

use crate::state::State;
use crate::types::Info;
use crate::util::pattern::pattern_in_scope;

/// Handle an inline code node.
///
/// Port of JS `lib/handle/inline-code.js`.
pub fn handle_inline_code(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    _info: &Info,
) -> String {
    if let Node::InlineCode(code) = node {
        let mut value = code.value.clone();
        let mut sequence = "`".to_string();

        // If there is a single grave accent on its own in the code, use a fence of two, etc.
        loop {
            let pattern = format!("(^|[^`]){}([^`]|$)", regex::escape(&sequence));
            if regex::Regex::new(&pattern).unwrap().is_match(&value) {
                sequence.push('`');
            } else {
                break;
            }
        }

        // If this is not just spaces or eols (tabs don't count), and either the
        // first or last character are a space, eol, or tick, then pad with spaces.
        let has_non_space = regex::Regex::new(r"[^ \r\n]").unwrap().is_match(&value);
        let starts_with_space_or_tick = regex::Regex::new(r"^[ \r\n`]").unwrap().is_match(&value);
        let ends_with_space_or_tick = regex::Regex::new(r"[ \r\n`]$").unwrap().is_match(&value);

        if has_non_space && (starts_with_space_or_tick || ends_with_space_or_tick) {
            value = format!(" {} ", value);
        }

        // Replace eols that could result in block constructs being seen.
        for pattern in &state.unsafe_patterns {
            if !pattern.at_break {
                continue;
            }
            if !pattern_in_scope(&state.stack, pattern) {
                continue;
            }

            let expression = crate::util::pattern::compile_pattern(pattern);
            while let Some(mat) = expression.find(&value) {
                let mut position = mat.start();

                // Support CRLF
                if value.as_bytes().get(position) == Some(&b'\n')
                    && position > 0
                    && value.as_bytes().get(position - 1) == Some(&b'\r')
                {
                    position -= 1;
                }

                value = format!(
                    "{} {}",
                    &value[..position],
                    &value[mat.start() + 1..]
                );
            }
        }

        format!("{}{}{}", sequence, value, sequence)
    } else {
        String::new()
    }
}

/// Peek function for inline code.
pub fn peek_inline_code(
    _node: &Node,
    _parent: Option<&Node>,
    _state: &mut State,
    _info: &Info,
) -> String {
    "`".to_string()
}
