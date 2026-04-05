use markdown::mdast::Node;

use crate::state::State;
use crate::types::{Info, TrackFields};
use crate::util::track::Tracker;

/// Serialize the children of a parent that contains flow children.
///
/// Port of JS `lib/util/container-flow.js`.
///
/// These children will typically be joined by blank lines.
/// What they are joined by exactly is defined by `Join` functions.
pub fn container_flow(parent: &Node, state: &mut State, info: &TrackFields) -> String {
    let children = get_children(parent);
    let mut tracker = Tracker::new(info);
    let mut results: Vec<String> = Vec::new();

    state.index_stack.push(0);

    for (index, child) in children.iter().enumerate() {
        let stack_len = state.index_stack.len();
        state.index_stack[stack_len - 1] = index;

        let child_info = Info {
            before: "\n".to_string(),
            after: "\n".to_string(),
            line: tracker.current().line,
            column: tracker.current().column,
            line_shift: tracker.current().line_shift,
        };

        let value = state.handle(child, Some(parent), &child_info);
        let moved = tracker.r#move(&value);
        results.push(moved);

        if !matches!(child, Node::List(_)) {
            state.bullet_last_used = None;
        }

        if index < children.len() - 1 {
            let between_str = between(child, &children[index + 1], parent, state);
            let moved = tracker.r#move(&between_str);
            results.push(moved);
        }
    }

    state.index_stack.pop();

    results.join("")
}

/// Determine what to put between two adjacent flow children.
///
/// Iterates join functions in reverse order. The first one that returns
/// a definitive result wins. Default is two newlines (one blank line).
fn between(left: &Node, right: &Node, parent: &Node, state: &State) -> String {
    let mut index = state.join.len();

    while index > 0 {
        index -= 1;
        let result = state.join[index](left, right, parent, state);

        match result {
            Some(1) => break,               // true or 1 in JS -> one blank line
            Some(n) if n >= 0 => {
                return "\n".repeat(1 + n as usize);
            }
            Some(_) => {
                // Negative (false in JS) -> cannot be adjacent, insert comment
                return "\n\n<!---->\n\n".to_string();
            }
            None => continue,               // No opinion
        }
    }

    "\n\n".to_string()
}

/// Get children from a flow parent node.
fn get_children(node: &Node) -> &[Node] {
    match node {
        Node::Root(root) => &root.children,
        Node::Blockquote(bq) => &bq.children,
        Node::List(list) => &list.children,
        Node::ListItem(li) => &li.children,
        Node::FootnoteDefinition(fd) => &fd.children,
        _ => &[],
    }
}
