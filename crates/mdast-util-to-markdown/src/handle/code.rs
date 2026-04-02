use markdown::mdast::Node;

use crate::state::State;
use crate::types::{ConstructName, Info, SafeConfig};
use crate::util::check::check_fence;
use crate::util::format::{format_code_as_indented, longest_streak};

/// Handle a code (fenced or indented) node.
///
/// Port of JS `lib/handle/code.js`.
pub fn handle_code(
    node: &Node,
    _parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::Code(code) = node {
        let marker = check_fence(&state.options);
        let raw = code.value.as_str();
        let suffix = if marker == '`' {
            "GraveAccent"
        } else {
            "Tilde"
        };

        if format_code_as_indented(code, state) {
            state.enter(ConstructName::CodeIndented);
            let value = state.indent_lines(raw, indented_code_map);
            state.exit();
            return value;
        }

        let mut tracker = state.create_tracker(info);
        let fence_count = longest_streak(raw, marker).max(2) + 1;
        let sequence: String = std::iter::repeat_n(marker, fence_count).collect();

        state.enter(ConstructName::CodeFenced);
        let mut value = tracker.r#move(&sequence);

        if let Some(ref lang) = code.lang {
            let construct = match suffix {
                "GraveAccent" => ConstructName::CodeFencedLangGraveAccent,
                _ => ConstructName::CodeFencedLangTilde,
            };
            state.enter(construct);
            let safe_lang = state.safe(
                Some(lang),
                &SafeConfig {
                    before: value.clone(),
                    after: " ".to_string(),
                    encode: vec!['`'],
                },
            );
            value += &tracker.r#move(&safe_lang);
            state.exit();
        }

        if code.lang.is_some() {
            if let Some(ref meta) = code.meta {
                let construct = match suffix {
                    "GraveAccent" => ConstructName::CodeFencedMetaGraveAccent,
                    _ => ConstructName::CodeFencedMetaTilde,
                };
                state.enter(construct);
                value += &tracker.r#move(" ");
                let safe_meta = state.safe(
                    Some(meta),
                    &SafeConfig {
                        before: value.clone(),
                        after: "\n".to_string(),
                        encode: vec!['`'],
                    },
                );
                value += &tracker.r#move(&safe_meta);
                state.exit();
            }
        }

        value += &tracker.r#move("\n");

        if !raw.is_empty() {
            value += &tracker.r#move(&format!("{}\n", raw));
        }

        value += &tracker.r#move(&sequence);

        state.exit(); // codeFenced

        value
    } else {
        String::new()
    }
}

/// Indent map function for indented code.
fn indented_code_map(line: &str, _index: usize, blank: bool) -> String {
    if blank {
        String::new()
    } else {
        format!("    {}", line)
    }
}
