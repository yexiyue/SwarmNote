use markdown::mdast::Node;

use crate::state::State;
use crate::types::Info;

/// Handle an HTML node.
///
/// Port of JS `lib/handle/html.js`.
pub fn handle_html(
    node: &Node,
    _parent: Option<&Node>,
    _state: &mut State,
    _info: &Info,
) -> String {
    if let Node::Html(html) = node {
        html.value.clone()
    } else {
        String::new()
    }
}

/// Peek function for html.
pub fn peek_html(
    _node: &Node,
    _parent: Option<&Node>,
    _state: &mut State,
    _info: &Info,
) -> String {
    "<".to_string()
}
