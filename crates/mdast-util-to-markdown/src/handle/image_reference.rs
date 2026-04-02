use markdown::mdast::{Node, ReferenceKind};

use crate::state::State;
use crate::types::{ConstructName, Info, SafeConfig};

/// Handle an image reference node.
///
/// Port of JS `lib/handle/image-reference.js`.
pub fn handle_image_reference(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::ImageReference(image_ref) = node {
        let ref_type = &image_ref.reference_kind;

        state.enter(ConstructName::ImageReference);
        state.enter(ConstructName::Label);
        let mut tracker = state.create_tracker(info);
        let mut value = tracker.r#move("![");

        let alt = state.safe(
            Some(&image_ref.alt),
            &SafeConfig {
                before: value.clone(),
                after: "]".to_string(),
                encode: vec![],
            },
        );
        value += &tracker.r#move(&format!("{}][", alt));

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
        state.exit(); // imageReference

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

/// Peek function for image reference.
pub fn peek_image_reference(
    _node: &Node,
    _parent: Option<&Node>,
    _state: &mut State,
    _info: &Info,
) -> String {
    "!".to_string()
}
