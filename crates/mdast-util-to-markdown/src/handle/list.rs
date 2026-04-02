use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, Info, TrackFields};
use crate::util::check::{check_bullet, check_bullet_ordered, check_bullet_other, check_rule};

/// Handle a list node.
///
/// Port of JS `lib/handle/list.js`.
pub fn handle_list(
    node: &Node,
    parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::List(list) = node {
        state.enter(ConstructName::List);
        let bullet_current = state.bullet_current.clone();

        let mut bullet = if list.ordered {
            check_bullet_ordered(&state.options).to_string()
        } else {
            check_bullet(&state.options).to_string()
        };

        let bullet_other = if list.ordered {
            if bullet == "." { ")".to_string() } else { ".".to_string() }
        } else {
            check_bullet_other(&state.options).to_string()
        };

        let mut use_different_marker = parent.is_some()
            && state.bullet_last_used.is_some()
            && bullet == *state.bullet_last_used.as_ref().unwrap();

        if !list.ordered {
            let first_list_item = list.children.first();

            // If there's an empty first list item directly in two list items,
            // we have to use a different bullet (to avoid thematic break).
            if bullet == "*" || bullet == "-" {
                if let Some(Node::ListItem(first_item)) = first_list_item {
                    if first_item.children.is_empty() {
                        let stack = &state.stack;
                        let idx_stack = &state.index_stack;
                        if stack.len() >= 4
                            && stack[stack.len() - 1] == ConstructName::List
                            && stack[stack.len() - 2] == ConstructName::ListItem
                            && stack[stack.len() - 3] == ConstructName::List
                            && stack[stack.len() - 4] == ConstructName::ListItem
                            && idx_stack.len() >= 3
                            && idx_stack[idx_stack.len() - 1] == 0
                            && idx_stack[idx_stack.len() - 2] == 0
                            && idx_stack[idx_stack.len() - 3] == 0
                        {
                            use_different_marker = true;
                        }
                    }
                }
            }

            // If there's a thematic break at the start of a list item, use different bullet.
            let rule = check_rule(&state.options).to_string();
            if rule == bullet {
                if let Some(_first) = first_list_item {
                    for child in &list.children {
                        if let Node::ListItem(item) = child {
                            if let Some(first_child) = item.children.first() {
                                if matches!(first_child, Node::ThematicBreak(_)) {
                                    use_different_marker = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if use_different_marker {
            bullet = bullet_other;
        }

        state.bullet_current = Some(bullet.clone());
        let track_fields = TrackFields {
            line: info.line,
            column: info.column,
            line_shift: info.line_shift,
        };
        let value = state.container_flow(node, &track_fields);
        state.bullet_last_used = Some(bullet);
        state.bullet_current = bullet_current;
        state.exit(); // list

        value
    } else {
        String::new()
    }
}
