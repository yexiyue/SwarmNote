use markdown::mdast::Node;

use crate::state::State;
use crate::types::Info;
use crate::util::pattern::pattern_in_scope;

/// Handle a break (hard break) node.
///
/// Port of JS `lib/handle/break.js`.
pub fn handle_break(
    _node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    for pattern in &state.unsafe_patterns {
        // If we can't put eols in this construct (setext headings, tables), use a
        // space instead.
        if pattern.character == '\n' && pattern_in_scope(&state.stack, pattern) {
            let before = &info.before;
            if before.ends_with(' ') || before.ends_with('\t') {
                return String::new();
            }
            return " ".to_string();
        }
    }

    "\\\n".to_string()
}
