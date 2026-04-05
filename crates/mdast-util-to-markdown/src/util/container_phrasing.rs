use markdown::mdast::Node;

use crate::state::State;
use crate::types::Info;
use crate::util::safe::encode_character_reference;
use crate::util::track::Tracker;

/// Serialize the children of a parent that contains phrasing children.
///
/// Port of JS `lib/util/container-phrasing.js`.
///
/// These children will be joined flush together.
pub fn container_phrasing(parent: &Node, state: &mut State, info: &Info) -> String {
    let children = get_phrasing_children(parent);
    let mut results: Vec<String> = Vec::new();
    let mut before = info.before.clone();
    let mut encode_after: Option<String> = None;

    state.index_stack.push(0);

    let mut tracker = Tracker::new(&crate::types::TrackFields {
        line: info.line,
        column: info.column,
        line_shift: info.line_shift,
    });

    for (index, child) in children.iter().enumerate() {
        let stack_len = state.index_stack.len();
        state.index_stack[stack_len - 1] = index;

        // Determine the `after` character for context.
        let after = if index + 1 < children.len() {
            let next_child = &children[index + 1];
            let peek_info = Info {
                before: String::new(),
                after: String::new(),
                line: tracker.current().line,
                column: tracker.current().column,
                line_shift: tracker.current().line_shift,
            };
            let peek_result = state.peek(next_child, Some(parent), &peek_info);
            if peek_result.is_empty() {
                String::new()
            } else {
                peek_result.chars().next().map_or(String::new(), |c| c.to_string())
            }
        } else {
            info.after.clone()
        };

        // In some cases, html (text) can be found in phrasing right after an eol.
        // When we'd serialize that, in most cases that would be seen as html (flow).
        // As we can't escape or so to prevent it from happening, we take a somewhat
        // reasonable approach: replace that eol with a space.
        if !results.is_empty()
            && (before == "\r" || before == "\n")
            && matches!(child, Node::Html(_))
        {
            if let Some(last) = results.last_mut() {
                let replaced = regex_replace_trailing_eol(last);
                *last = replaced;
            }
            before = " ".to_string();

            // Reset tracker
            tracker = Tracker::new(&crate::types::TrackFields {
                line: info.line,
                column: info.column,
                line_shift: info.line_shift,
            });
            let joined = results.join("");
            tracker.r#move(&joined);
        }

        let child_info = Info {
            before: before.clone(),
            after: after.clone(),
            line: tracker.current().line,
            column: tracker.current().column,
            line_shift: tracker.current().line_shift,
        };

        let mut value = state.handle(child, Some(parent), &child_info);

        // If we had to encode the first character after the previous node and it's
        // still the same character, encode it.
        if let Some(ref enc) = encode_after {
            if value.starts_with(enc.as_str()) {
                let code = enc.chars().next().unwrap() as u32;
                value = format!(
                    "{}{}",
                    encode_character_reference(code),
                    &value[enc.len()..]
                );
            }
        }

        let encoding_info = state.attention_encode_surrounding_info.take();
        encode_after = None;

        // Handle attention encoding info.
        if let Some(ref enc_info) = encoding_info {
            if !results.is_empty() && enc_info.before {
                if let Some(last) = results.last_mut() {
                    if before == last.chars().last().map_or(String::new(), |c| c.to_string()) {
                        let last_char = last.chars().last().unwrap();
                        let encoded = encode_character_reference(last_char as u32);
                        let trimmed = &last[..last.len() - last_char.len_utf8()];
                        *last = format!("{}{}", trimmed, encoded);
                    }
                }
            }

            if enc_info.after {
                encode_after = Some(after.clone());
            }
        }

        tracker.r#move(&value);
        results.push(value.clone());
        before = value.chars().last().map_or(String::new(), |c| c.to_string());
    }

    state.index_stack.pop();

    results.join("")
}

/// Replace trailing line ending with a space.
fn regex_replace_trailing_eol(s: &str) -> String {
    // Matches \r\n, \r, or \n at the end of the string
    if let Some(stripped) = s.strip_suffix("\r\n") {
        format!("{} ", stripped)
    } else if let Some(stripped) = s.strip_suffix('\r').or_else(|| s.strip_suffix('\n')) {
        format!("{} ", stripped)
    } else {
        s.to_string()
    }
}

/// Get phrasing children from a parent node.
fn get_phrasing_children(node: &Node) -> &[Node] {
    match node {
        Node::Root(r) => &r.children,
        Node::Paragraph(p) => &p.children,
        Node::Heading(h) => &h.children,
        Node::Emphasis(e) => &e.children,
        Node::Strong(s) => &s.children,
        Node::Delete(d) => &d.children,
        Node::Link(l) => &l.children,
        Node::LinkReference(lr) => &lr.children,
        Node::TableCell(tc) => &tc.children,
        _ => &[],
    }
}
