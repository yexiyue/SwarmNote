use markdown::mdast::Node;

use crate::state::State;
use crate::types::{Info, SafeConfig};

/// Handle a text node.
///
/// Port of JS `lib/handle/text.js`.
pub fn handle_text(node: &Node, _parent: Option<&Node>, state: &mut State, info: &Info) -> String {
    if let Node::Text(text) = node {
        state.safe(
            Some(&text.value),
            &SafeConfig {
                before: info.before.clone(),
                after: info.after.clone(),
                encode: vec![],
            },
        )
    } else {
        String::new()
    }
}
