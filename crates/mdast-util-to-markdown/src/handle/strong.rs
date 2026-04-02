use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, EncodeSurrounding, Info};
use crate::util::check::check_strong;
use crate::util::format::encode_info;
use crate::util::safe::encode_character_reference;

/// Handle a strong node.
///
/// Port of JS `lib/handle/strong.js`.
pub fn handle_strong(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    let marker = check_strong(&state.options);
    let double_marker = format!("{}{}", marker, marker);

    state.enter(ConstructName::Strong);
    let mut tracker = state.create_tracker(info);
    let before = tracker.r#move(&double_marker);

    let mut between = {
        let phrasing_info = Info {
            after: marker.to_string(),
            before: before.clone(),
            line: tracker.current().line,
            column: tracker.current().column,
            line_shift: tracker.current().line_shift,
        };
        let v = state.container_phrasing(node, &phrasing_info);
        tracker.r#move(&v)
    };

    let between_head = between.chars().next();
    let info_before_last = info.before.chars().last();

    let open = encode_info(info_before_last, between_head, marker);

    if open.inside {
        if let Some(head) = between_head {
            between = format!(
                "{}{}",
                encode_character_reference(head as u32),
                &between[head.len_utf8()..]
            );
        }
    }

    let between_tail = between.chars().last();
    let info_after_first = info.after.chars().next();

    let close = encode_info(info_after_first, between_tail, marker);

    if close.inside {
        if let Some(tail) = between_tail {
            let trimmed = &between[..between.len() - tail.len_utf8()];
            between = format!("{}{}", trimmed, encode_character_reference(tail as u32));
        }
    }

    let after = tracker.r#move(&double_marker);

    state.exit(); // strong

    state.attention_encode_surrounding_info = Some(EncodeSurrounding {
        after: close.outside,
        before: open.outside,
    });

    format!("{}{}{}", before, between, after)
}

/// Peek function for strong.
pub fn peek_strong(
    _node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    _info: &Info,
) -> String {
    state.options.strong.to_string()
}
