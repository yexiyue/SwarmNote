use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, Info};

/// Handle a delete (strikethrough) node.
///
/// Port of JS `mdast-util-gfm-strikethrough` toMarkdown handler.
pub fn handle_delete(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    let mut tracker = state.create_tracker(info);
    state.enter(ConstructName::Strikethrough);
    let mut value = tracker.r#move("~~");
    let phrasing_info = Info {
        before: value.clone(),
        after: "~".to_string(),
        line: tracker.current().line,
        column: tracker.current().column,
        line_shift: tracker.current().line_shift,
    };
    value += &state.container_phrasing(node, &phrasing_info);
    value += &tracker.r#move("~~");
    state.exit(); // strikethrough
    value
}

/// Peek function for delete (strikethrough).
pub fn peek_delete(
    _node: &Node,
    _parent: Option<&Node>,
    _state: &mut State,
    _info: &Info,
) -> String {
    "~".to_string()
}
