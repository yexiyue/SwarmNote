use markdown::mdast::{Node, ReferenceKind};

use crate::state::State;
use crate::types::{ConstructName, Info, SafeConfig};

/// Handle a link reference node.
///
/// Port of JS `lib/handle/link-reference.js`.
pub fn handle_link_reference(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::LinkReference(link_ref) = node {
        let ref_type = &link_ref.reference_kind;

        state.enter(ConstructName::LinkReference);
        state.enter(ConstructName::Label);
        let mut tracker = state.create_tracker(info);
        let mut value = tracker.r#move("[");

        let text = state.container_phrasing(node, &Info {
            before: value.clone(),
            after: "]".to_string(),
            line: tracker.current().line,
            column: tracker.current().column,
            line_shift: tracker.current().line_shift,
        });
        value += &tracker.r#move(&format!("{}][", text));

        state.exit(); // label

        // Hide the fact that we're in phrasing, because escapes don't work.
        let saved_stack = std::mem::take(&mut state.stack);
        state.enter(ConstructName::Reference);

        let id = state.association_id(node);
        let reference = state.safe(
            Some(&id),
            &SafeConfig {
                before: value.clone(),
                after: "]".to_string(),
                encode: vec![],
            },
        );

        state.exit(); // reference
        state.stack = saved_stack;
        state.exit(); // linkReference

        match ref_type {
            ReferenceKind::Full => {
                value += &tracker.r#move(&format!("{}]", reference));
            }
            ReferenceKind::Shortcut => {
                // Remove the unwanted `[`.
                value = value[..value.len() - 1].to_string();
            }
            ReferenceKind::Collapsed => {
                value += &tracker.r#move("]");
            }
        }

        value
    } else {
        String::new()
    }
}

/// Peek function for link reference.
pub fn peek_link_reference(
    _node: &Node,
    _parent: Option<&Node>,
    _state: &mut State,
    _info: &Info,
) -> String {
    "[".to_string()
}
