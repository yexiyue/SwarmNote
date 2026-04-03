use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, Info};

/// Handle a blockquote node.
///
/// Port of JS `lib/handle/blockquote.js`.
pub fn handle_blockquote(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    state.enter(ConstructName::Blockquote);
    let mut tracker = state.create_tracker(info);
    tracker.r#move("> ");
    tracker.shift(2);
    let flow_result = state.container_flow(node, &tracker.current());
    let value = state.indent_lines(&flow_result, |line, _index, blank| {
        format!(">{}{}", if blank { "" } else { " " }, line)
    });
    state.exit(); // blockquote
    value
}
