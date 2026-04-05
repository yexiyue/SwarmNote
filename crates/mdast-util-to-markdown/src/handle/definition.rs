use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, Info, SafeConfig};
use crate::util::check::check_quote;

/// Handle a definition node.
///
/// Port of JS `lib/handle/definition.js`.
pub fn handle_definition(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::Definition(def) = node {
        let quote = check_quote(&state.options);
        let suffix = if quote == '"' { "Quote" } else { "Apostrophe" };

        state.enter(ConstructName::Definition);
        state.enter(ConstructName::Label);
        let mut tracker = state.create_tracker(info);
        let mut value = tracker.r#move("[");

        let id = state.association_id(node);
        let safe_id = state.safe(
            Some(&id),
            &SafeConfig {
                before: value.clone(),
                after: "]".to_string(),
                encode: vec![],
            },
        );
        value += &tracker.r#move(&safe_id);
        value += &tracker.r#move("]: ");
        state.exit(); // label

        let needs_angle = def.url.is_empty()
            || regex::Regex::new(r"[\x00-\x20\x7F]")
                .unwrap()
                .is_match(&def.url);

        if needs_angle {
            state.enter(ConstructName::DestinationLiteral);
            value += &tracker.r#move("<");
            let safe_url = state.safe(
                Some(&def.url),
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
            let after = if def.title.is_some() {
                " ".to_string()
            } else {
                "\n".to_string()
            };
            let safe_url = state.safe(
                Some(&def.url),
                &SafeConfig {
                    before: value.clone(),
                    after,
                    encode: vec![],
                },
            );
            value += &tracker.r#move(&safe_url);
        }
        state.exit(); // destination

        if let Some(ref title) = def.title {
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

        state.exit(); // definition

        value
    } else {
        String::new()
    }
}
