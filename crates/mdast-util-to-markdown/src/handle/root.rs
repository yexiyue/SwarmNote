use markdown::mdast::Node;

use crate::state::State;
use crate::types::{Info, TrackFields};
use crate::util::format::is_phrasing;

/// Handle a root node.
///
/// Port of JS `lib/handle/root.js`.
///
/// If the root contains phrasing children, serialize as phrasing.
/// Otherwise serialize as flow.
pub fn handle_root(node: &Node, _parent: Option<&Node>, state: &mut State, info: &Info) -> String {
    if let Node::Root(root) = node {
        let has_phrasing = root.children.iter().any(is_phrasing);

        if has_phrasing {
            state.container_phrasing(node, info)
        } else {
            let track_fields = TrackFields {
                line: info.line,
                column: info.column,
                line_shift: info.line_shift,
            };
            state.container_flow(node, &track_fields)
        }
    } else {
        String::new()
    }
}
