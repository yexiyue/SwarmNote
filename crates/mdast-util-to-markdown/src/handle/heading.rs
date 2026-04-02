use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, Info};
use crate::util::format::format_heading_as_setext;
use crate::util::safe::encode_character_reference;

/// Handle a heading node.
///
/// Port of JS `lib/handle/heading.js`.
pub fn handle_heading(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::Heading(heading) = node {
        let rank = heading.depth.clamp(1, 6) as usize;
        let mut tracker = state.create_tracker(info);

        if format_heading_as_setext(heading, state) {
            state.enter(ConstructName::HeadingSetext);
            state.enter(ConstructName::Phrasing);
            let value = state.container_phrasing(node, &Info {
                before: "\n".to_string(),
                after: "\n".to_string(),
                line: tracker.current().line,
                column: tracker.current().column,
                line_shift: tracker.current().line_shift,
            });
            state.exit(); // phrasing
            state.exit(); // headingSetext

            let marker = if rank == 1 { "=" } else { "-" };
            // Length is from after the last EOL (or 0 if none)
            let last_eol = value
                .rfind('\r')
                .map(|i| i + 1)
                .unwrap_or(0)
                .max(value.rfind('\n').map(|i| i + 1).unwrap_or(0));
            let underline_len = value.len() - last_eol;

            return format!("{}\n{}", value, marker.repeat(underline_len));
        }

        let sequence = "#".repeat(rank);
        state.enter(ConstructName::HeadingAtx);
        state.enter(ConstructName::Phrasing);

        tracker.r#move(&format!("{} ", sequence));

        let mut value = state.container_phrasing(node, &Info {
            before: "# ".to_string(),
            after: "\n".to_string(),
            line: tracker.current().line,
            column: tracker.current().column,
            line_shift: tracker.current().line_shift,
        });

        if value.starts_with('\t') || value.starts_with(' ') {
            if let Some(first_char) = value.chars().next() {
                value = format!(
                    "{}{}",
                    encode_character_reference(first_char as u32),
                    &value[first_char.len_utf8()..]
                );
            }
        }

        let mut result = if value.is_empty() {
            sequence.clone()
        } else {
            format!("{} {}", sequence, value)
        };

        if state.options.close_atx {
            result = format!("{} {}", result, sequence);
        }

        state.exit(); // phrasing
        state.exit(); // headingAtx

        result
    } else {
        String::new()
    }
}
