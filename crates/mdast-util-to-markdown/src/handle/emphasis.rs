use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, EncodeSurrounding, Info};
use crate::util::check::check_emphasis;
use crate::util::format::encode_info;
use crate::util::safe::encode_character_reference;

/// Handle an emphasis node.
///
/// Port of JS `lib/handle/emphasis.js`.
pub fn handle_emphasis(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    let marker = check_emphasis(&state.options);
    let marker_str = marker.to_string();

    state.enter(ConstructName::Emphasis);
    let mut tracker = state.create_tracker(info);
    let before = tracker.r#move(&marker_str);

    let mut between = {
        let phrasing_info = Info {
            after: marker_str.clone(),
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

    let after = tracker.r#move(&marker_str);

    state.exit(); // emphasis

    state.attention_encode_surrounding_info = Some(EncodeSurrounding {
        after: close.outside,
        before: open.outside,
    });

    format!("{}{}{}", before, between, after)
}

/// Peek function for emphasis.
pub fn peek_emphasis(
    _node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    _info: &Info,
) -> String {
    state.options.emphasis.to_string()
}
