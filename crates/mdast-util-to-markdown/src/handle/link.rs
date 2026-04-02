use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, Info, SafeConfig};
use crate::util::check::check_quote;
use crate::util::format::format_link_as_autolink;

/// Handle a link node.
///
/// Port of JS `lib/handle/link.js`.
pub fn handle_link(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::Link(link) = node {
        let quote = check_quote(&state.options);
        let suffix = if quote == '"' { "Quote" } else { "Apostrophe" };
        let mut tracker = state.create_tracker(info);

        if format_link_as_autolink(link, state) {
            // Hide the fact that we're in phrasing, because escapes don't work.
            let saved_stack = std::mem::take(&mut state.stack);
            state.enter(ConstructName::Autolink);
            let mut value = tracker.r#move("<");
            let phrasing_info = Info {
                before: value.clone(),
                after: ">".to_string(),
                line: tracker.current().line,
                column: tracker.current().column,
                line_shift: tracker.current().line_shift,
            };
            value += &tracker.r#move(&state.container_phrasing(node, &phrasing_info));
            value += &tracker.r#move(">");
            state.exit(); // autolink
            state.stack = saved_stack;
            return value;
        }

        state.enter(ConstructName::Link);
        state.enter(ConstructName::Label);
        let mut value = tracker.r#move("[");
        let phrasing_info = Info {
            before: value.clone(),
            after: "](".to_string(),
            line: tracker.current().line,
            column: tracker.current().column,
            line_shift: tracker.current().line_shift,
        };
        value += &tracker.r#move(&state.container_phrasing(node, &phrasing_info));
        value += &tracker.r#move("](");
        state.exit(); // label

        // Check if URL needs angle brackets
        let needs_angle = (!link.url.is_empty()
            && link.title.is_some()
            && link.url.is_empty())
            || regex::Regex::new(r"[\x00-\x20\x7F]")
                .unwrap()
                .is_match(&link.url);

        if (link.url.is_empty() && link.title.is_some()) || needs_angle {
            state.enter(ConstructName::DestinationLiteral);
            value += &tracker.r#move("<");
            let safe_url = state.safe(
                Some(&link.url),
                &SafeConfig {
                    before: value.clone(),
                    after: ">".to_string(),
                    encode: vec![],
                },
            );
            value += &tracker.r#move(&safe_url);
            value += &tracker.r#move(">");
        } else {
            state.enter(ConstructName::DestinationRaw);
            let after = if link.title.is_some() {
                " ".to_string()
            } else {
                ")".to_string()
            };
            let safe_url = state.safe(
                Some(&link.url),
                &SafeConfig {
                    before: value.clone(),
                    after,
                    encode: vec![],
                },
            );
            value += &tracker.r#move(&safe_url);
        }
        state.exit(); // destination

        if let Some(ref title) = link.title {
            let title_construct = if suffix == "Quote" {
                ConstructName::TitleQuote
            } else {
                ConstructName::TitleApostrophe
            };
            state.enter(title_construct);
            value += &tracker.r#move(&format!(" {}", quote));
            let safe_title = state.safe(
                Some(title),
                &SafeConfig {
                    before: value.clone(),
                    after: quote.to_string(),
                    encode: vec![],
                },
            );
            value += &tracker.r#move(&safe_title);
            value += &tracker.r#move(&quote.to_string());
            state.exit(); // title
        }

        value += &tracker.r#move(")");

        state.exit(); // link

        value
    } else {
        String::new()
    }
}

/// Peek function for link.
pub fn peek_link(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    _info: &Info,
) -> String {
    if let Node::Link(link) = node {
        if format_link_as_autolink(link, state) {
            "<".to_string()
        } else {
            "[".to_string()
        }
    } else {
        "[".to_string()
    }
}
