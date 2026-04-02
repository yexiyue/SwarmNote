use markdown::mdast::Node;

use crate::state::State;
use crate::types::JoinFn;
use crate::util::format::{format_code_as_indented, format_heading_as_setext};

/// Default join functions.
///
/// Port of JS `lib/join.js`.
pub fn default_join() -> Vec<JoinFn> {
    vec![join_defaults]
}

/// Default join function.
///
/// Handles:
/// - Indented code after list or another indented code (cannot be adjacent)
/// - Children of a list or an item (respects `spread` field)
fn join_defaults(left: &Node, right: &Node, parent: &Node, state: &State) -> Option<i32> {
    // Indented code after list or another indented code.
    if let Node::Code(right_code) = right {
        if format_code_as_indented(right_code, state) {
            if matches!(left, Node::List(_)) {
                return Some(-1); // false in JS -> cannot be adjacent
            }
            if let Node::Code(left_code) = left {
                if format_code_as_indented(left_code, state) {
                    return Some(-1); // false in JS -> cannot be adjacent
                }
            }
        }
    }

    // Join children of a list or an item.
    // In which case, `parent` has a `spread` field.
    let spread = match parent {
        Node::List(list) => Some(list.spread),
        Node::ListItem(item) => Some(item.spread),
        _ => None,
    };

    if let Some(spread) = spread {
        if matches!(left, Node::Paragraph(_)) {
            let is_paragraph_pair = matches!(right, Node::Paragraph(_));
            let is_definition = matches!(right, Node::Definition(_));
            let is_setext_heading = if let Node::Heading(h) = right {
                format_heading_as_setext(h, state)
            } else {
                false
            };

            if is_paragraph_pair || is_definition || is_setext_heading {
                // Return None (undefined in JS) - no opinion
                return None;
            }
        }

        return Some(if spread { 1 } else { 0 });
    }

    None
}
