use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, Info, SafeConfig};
use crate::util::check::check_quote;

/// Handle an image node.
///
/// Port of JS `lib/handle/image.js`.
pub fn handle_image(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::Image(image) = node {
        let quote = check_quote(&state.options);
        let suffix = if quote == '"' { "Quote" } else { "Apostrophe" };

        state.enter(ConstructName::Image);
        state.enter(ConstructName::Label);
        let mut tracker = state.create_tracker(info);
        let mut value = tracker.r#move("![");
        let safe_alt = state.safe(
            Some(&image.alt),
            &SafeConfig {
                before: value.clone(),
                after: "]".to_string(),
                encode: vec![],
            },
        );
        value += &tracker.r#move(&safe_alt);
        value += &tracker.r#move("](");
        state.exit(); // label

        let needs_angle = (image.url.is_empty() && image.title.is_some())
            || regex::Regex::new(r"[\x00-\x20\x7F]")
                .unwrap()
                .is_match(&image.url);

        if needs_angle {
            state.enter(ConstructName::DestinationLiteral);
            value += &tracker.r#move("<");
            let safe_url = state.safe(
                Some(&image.url),
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
            let after = if image.title.is_some() {
                " ".to_string()
            } else {
                ")".to_string()
            };
            let safe_url = state.safe(
                Some(&image.url),
                &SafeConfig {
                    before: value.clone(),
                    after,
                    encode: vec![],
                },
            );
            value += &tracker.r#move(&safe_url);
        }
        state.exit(); // destination

        if let Some(ref title) = image.title {
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
        state.exit(); // image

        value
    } else {
        String::new()
    }
}

/// Peek function for image.
pub fn peek_image(
    _node: &Node,
    _parent: Option<&Node>,
    _state: &mut State,
    _info: &Info,
) -> String {
    "!".to_string()
}
