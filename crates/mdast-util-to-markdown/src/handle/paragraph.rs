use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, Info};

/// Handle a paragraph node.
///
/// Port of JS `lib/handle/paragraph.js`.
pub fn handle_paragraph(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    state.enter(ConstructName::Paragraph);
    state.enter(ConstructName::Phrasing);
    let value = state.container_phrasing(node, info);
    state.exit(); // phrasing
    state.exit(); // paragraph
    value
}
