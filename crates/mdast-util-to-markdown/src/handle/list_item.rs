use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, Info, TrackFields};
use crate::util::check::check_bullet;
use crate::util::format::check_list_item_indent;
use crate::util::indent::indent_lines;
use crate::util::track::Tracker;

/// Handle a list item node.
///
/// Port of JS `lib/handle/list-item.js`.
pub fn handle_list_item(
    node: &Node,
    parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::ListItem(_list_item) = node {
        let list_item_indent = check_list_item_indent(state);
        let mut bullet = state
            .bullet_current
            .clone()
            .unwrap_or_else(|| check_bullet(&state.options).to_string());

        // Add the marker value for ordered lists.
        if let Some(Node::List(list)) = parent {
            if list.ordered {
                let start = list.start.unwrap_or(1) as usize;
                let index = list
                    .children
                    .iter()
                    .position(|c| std::ptr::eq(c, node))
                    .unwrap_or(0);
                let number = if state.options.increment_list_marker {
                    start + index
                } else {
                    start
                };
                bullet = format!("{}{}", number, bullet);
            }
        }

        let mut size = bullet.len() + 1;

        if list_item_indent == "tab"
            || (list_item_indent == "mixed" && is_spread(node, parent))
        {
            size = size.div_ceil(4) * 4; // ceil to next tab stop
        }

        let mut tracker = Tracker::new(&TrackFields {
            line: info.line,
            column: info.column,
            line_shift: info.line_shift,
        });
        let padding: String = " ".repeat(size - bullet.len());
        tracker.r#move(&format!("{}{}", bullet, padding));
        tracker.shift(size);

        state.enter(ConstructName::ListItem);
        let flow_result = state.container_flow(node, &tracker.current());
        let bullet_clone = bullet.clone();
        let value = indent_lines(&flow_result, move |line, index, blank| {
            if index > 0 {
                if blank {
                    String::new()
                } else {
                    " ".repeat(size) + line
                }
            } else if blank {
                bullet_clone.clone() + line
            } else {
                format!("{}{}{}", bullet_clone, " ".repeat(size - bullet_clone.len()), line)
            }
        });
        state.exit(); // listItem

        value
    } else {
        String::new()
    }
}

/// Check if a list item or its parent list is spread.
fn is_spread(node: &Node, parent: Option<&Node>) -> bool {
    if let Node::ListItem(item) = node {
        if item.spread {
            return true;
        }
    }
    if let Some(Node::List(list)) = parent {
        if list.spread {
            return true;
        }
    }
    false
}
