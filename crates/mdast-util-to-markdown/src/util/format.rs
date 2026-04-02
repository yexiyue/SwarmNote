use markdown::mdast::{Code, Heading, Node};

use crate::state::State;

/// Check whether a code node should be formatted as indented code.
///
/// Port of JS `lib/util/format-code-as-indented.js`.
///
/// Indented code is used when:
/// - `options.fences` is `false`
/// - The code has a value
/// - There's no language info
/// - There's a non-whitespace character
/// - The value doesn't start or end in a blank line
pub fn format_code_as_indented(node: &Code, state: &State) -> bool {
    if state.options.fences {
        return false;
    }

    let value = match &node.value {
        v if !v.is_empty() => v.as_str(),
        _ => return false,
    };

    // If there's a language, don't use indented
    if node.lang.is_some() {
        return false;
    }

    // Check there's a non-whitespace character
    let has_non_whitespace = value.chars().any(|c| c != ' ' && c != '\r' && c != '\n');
    if !has_non_whitespace {
        return false;
    }

    // Check value doesn't start or end in a blank line
    let starts_blank = regex::Regex::new(r"^[\t ]*(?:[\r\n]|$)").unwrap();
    let ends_blank = regex::Regex::new(r"(?:^|[\r\n])[\t ]*$").unwrap();

    if starts_blank.is_match(value) || ends_blank.is_match(value) {
        return false;
    }

    true
}

/// Check whether a heading should be formatted as setext.
///
/// Port of JS `lib/util/format-heading-as-setext.js`.
///
/// Setext headings are used when:
/// - `options.setext` is `true` (or there's a literal with a line break)
/// - The heading depth is 1 or 2
/// - The heading has text content
pub fn format_heading_as_setext(node: &Heading, state: &State) -> bool {
    let depth = node.depth;

    // Setext only works for h1 and h2
    if depth > 2 {
        return false;
    }

    // Must have content
    let content = node_to_string(&Node::Heading(node.clone()));
    if content.is_empty() {
        return false;
    }

    // Check for literal with line break
    let mut literal_with_break = false;
    visit_node(&Node::Heading(node.clone()), &mut |n| {
        match n {
            Node::Text(text) if text.value.contains('\n') || text.value.contains('\r') => {
                literal_with_break = true;
            }
            Node::InlineCode(code) if code.value.contains('\n') || code.value.contains('\r') => {
                literal_with_break = true;
            }
            Node::Break(_) => {
                literal_with_break = true;
            }
            _ => {}
        }
    });

    state.options.setext || literal_with_break
}

/// Convert a node to its plain text string content.
///
/// Port of JS `mdast-util-to-string`.
pub fn node_to_string(node: &Node) -> String {
    match node {
        Node::Text(text) => text.value.clone(),
        Node::InlineCode(code) => code.value.clone(),
        Node::Html(html) => html.value.clone(),
        Node::Code(code) => code.value.clone(),
        _ => {
            let children = get_all_children(node);
            children.iter().map(node_to_string).collect()
        }
    }
}

/// Visit all nodes in a tree, calling `f` for each.
fn visit_node(node: &Node, f: &mut dyn FnMut(&Node)) {
    f(node);
    for child in get_all_children(node) {
        visit_node(child, f);
    }
}

/// Get children of any node type.
fn get_all_children(node: &Node) -> &[Node] {
    match node {
        Node::Root(n) => &n.children,
        Node::Blockquote(n) => &n.children,
        Node::FootnoteDefinition(n) => &n.children,
        Node::List(n) => &n.children,
        Node::ListItem(n) => &n.children,
        Node::Emphasis(n) => &n.children,
        Node::Strong(n) => &n.children,
        Node::Delete(n) => &n.children,
        Node::Link(n) => &n.children,
        Node::Heading(n) => &n.children,
        Node::Table(n) => &n.children,
        Node::TableRow(n) => &n.children,
        Node::TableCell(n) => &n.children,
        Node::Paragraph(n) => &n.children,
        _ => &[],
    }
}

/// Check if a node is a "phrasing" node (inline content).
///
/// Port of JS `mdast-util-phrasing`.
pub fn is_phrasing(node: &Node) -> bool {
    matches!(
        node,
        Node::Break(_)
            | Node::Delete(_)
            | Node::Emphasis(_)
            | Node::FootnoteReference(_)
            | Node::Html(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::InlineCode(_)
            | Node::InlineMath(_)
            | Node::Link(_)
            | Node::LinkReference(_)
            | Node::Strong(_)
            | Node::Text(_)
    )
}

/// Find the longest streak of `character` in `value`.
///
/// Port of JS `longest-streak`.
pub fn longest_streak(value: &str, character: char) -> usize {
    let mut count = 0;
    let mut max = 0;

    for ch in value.chars() {
        if ch == character {
            count += 1;
            if count > max {
                max = count;
            }
        } else {
            count = 0;
        }
    }

    max
}

/// Classify a character as whitespace, punctuation, or other.
///
/// Port of JS `micromark-util-classify-character`.
#[derive(Debug, PartialEq)]
pub enum CharacterKind {
    Whitespace,
    Punctuation,
    Other,
}

pub fn classify_character(ch: char) -> CharacterKind {
    if ch.is_whitespace() || ch == '\t' || ch == '\n' || ch == '\r' || ch == ' ' {
        CharacterKind::Whitespace
    } else if ch.is_ascii_punctuation() || unicode_punctuation(ch) {
        CharacterKind::Punctuation
    } else {
        CharacterKind::Other
    }
}

/// Check if a character is Unicode punctuation (beyond ASCII).
fn unicode_punctuation(ch: char) -> bool {
    // General Unicode punctuation categories
    let cat = unicode_category(ch);
    matches!(
        cat,
        UnicodeCategory::Pc
            | UnicodeCategory::Pd
            | UnicodeCategory::Pe
            | UnicodeCategory::Pf
            | UnicodeCategory::Pi
            | UnicodeCategory::Po
            | UnicodeCategory::Ps
    )
}

#[derive(Debug, PartialEq)]
enum UnicodeCategory {
    Pc, // Connector punctuation
    Pd, // Dash punctuation
    Pe, // Close punctuation
    Pf, // Final punctuation
    Pi, // Initial punctuation
    Po, // Other punctuation
    Ps, // Open punctuation
    Other,
}

fn unicode_category(ch: char) -> UnicodeCategory {
    // Simplified check for common punctuation ranges
    // This covers the most common Unicode punctuation characters
    match ch {
        '_' => UnicodeCategory::Pc,
        '-' | '\u{2010}'..='\u{2015}' | '\u{2E17}' | '\u{2E1A}' | '\u{2E3A}'..='\u{2E3B}'
        | '\u{301C}' | '\u{3030}' | '\u{30A0}' | '\u{FE31}'..='\u{FE32}'
        | '\u{FE58}' | '\u{FE63}' | '\u{FF0D}' => UnicodeCategory::Pd,
        ')' | ']' | '}' | '\u{0F3B}' | '\u{0F3D}' | '\u{169C}' | '\u{2046}'
        | '\u{207E}' | '\u{208E}' | '\u{2309}' | '\u{230B}' | '\u{232A}'
        | '\u{2769}' | '\u{276B}' | '\u{276D}' | '\u{276F}' | '\u{2771}'
        | '\u{2773}' | '\u{2775}' | '\u{27C6}' | '\u{27E7}' | '\u{27E9}'
        | '\u{27EB}' | '\u{27ED}' | '\u{27EF}' | '\u{2984}' | '\u{2986}'
        | '\u{2988}' | '\u{298A}' | '\u{298C}' | '\u{298E}' | '\u{2990}'
        | '\u{2992}' | '\u{2994}' | '\u{2996}' | '\u{2998}' | '\u{29D9}'
        | '\u{29DB}' | '\u{29FD}' | '\u{2E23}' | '\u{2E25}' | '\u{2E27}'
        | '\u{2E29}' | '\u{3009}' | '\u{300B}' | '\u{300D}' | '\u{300F}'
        | '\u{3011}' | '\u{3015}' | '\u{3017}' | '\u{3019}' | '\u{301B}'
        | '\u{301E}'..='\u{301F}' | '\u{FD3E}' | '\u{FE18}' | '\u{FE36}'
        | '\u{FE38}' | '\u{FE3A}' | '\u{FE3C}' | '\u{FE3E}' | '\u{FE40}'
        | '\u{FE42}' | '\u{FE44}' | '\u{FE48}' | '\u{FE5A}' | '\u{FE5C}'
        | '\u{FE5E}' | '\u{FF09}' | '\u{FF3D}' | '\u{FF5D}' | '\u{FF60}'
        | '\u{FF63}' => UnicodeCategory::Pe,
        '\u{00BB}' | '\u{2019}' | '\u{201D}' | '\u{203A}' | '\u{2E03}'
        | '\u{2E05}' | '\u{2E0A}' | '\u{2E0D}' | '\u{2E1D}' | '\u{2E21}' => {
            UnicodeCategory::Pf
        }
        '\u{00AB}' | '\u{2018}' | '\u{201B}'..='\u{201C}' | '\u{201F}'
        | '\u{2039}' | '\u{2E02}' | '\u{2E04}' | '\u{2E09}' | '\u{2E0C}'
        | '\u{2E1C}' | '\u{2E20}' => UnicodeCategory::Pi,
        '(' | '[' | '{' | '\u{0F3A}' | '\u{0F3C}' | '\u{169B}' | '\u{2045}'
        | '\u{207D}' | '\u{208D}' | '\u{2308}' | '\u{230A}' | '\u{2329}'
        | '\u{2768}' | '\u{276A}' | '\u{276C}' | '\u{276E}' | '\u{2770}'
        | '\u{2772}' | '\u{2774}' | '\u{27C5}' | '\u{27E6}' | '\u{27E8}'
        | '\u{27EA}' | '\u{27EC}' | '\u{27EE}' | '\u{2983}' | '\u{2985}'
        | '\u{2987}' | '\u{2989}' | '\u{298B}' | '\u{298D}' | '\u{298F}'
        | '\u{2991}' | '\u{2993}' | '\u{2995}' | '\u{2997}' | '\u{29D8}'
        | '\u{29DA}' | '\u{29FC}' | '\u{2E22}' | '\u{2E24}' | '\u{2E26}'
        | '\u{2E28}' | '\u{3008}' | '\u{300A}' | '\u{300C}' | '\u{300E}'
        | '\u{3010}' | '\u{3014}' | '\u{3016}' | '\u{3018}' | '\u{301A}'
        | '\u{301D}' | '\u{FD3F}' | '\u{FE17}' | '\u{FE35}' | '\u{FE37}'
        | '\u{FE39}' | '\u{FE3B}' | '\u{FE3D}' | '\u{FE3F}' | '\u{FE41}'
        | '\u{FE43}' | '\u{FE47}' | '\u{FE59}' | '\u{FE5B}' | '\u{FE5D}'
        | '\u{FF08}' | '\u{FF3B}' | '\u{FF5B}' | '\u{FF5F}'
        | '\u{FF62}' => UnicodeCategory::Ps,
        '!' | '"' | '#' | '%' | '&' | '\'' | '*' | ',' | '.' | '/' | ':'
        | ';' | '?' | '@' | '\\' | '\u{00A1}' | '\u{00A7}' | '\u{00B6}'
        | '\u{00B7}' | '\u{00BF}' | '\u{037E}' | '\u{0387}'
        | '\u{055A}'..='\u{055F}' | '\u{0589}' | '\u{05C0}' | '\u{05C3}'
        | '\u{05C6}' | '\u{05F3}'..='\u{05F4}' | '\u{0609}'..='\u{060A}'
        | '\u{060C}'..='\u{060D}' | '\u{061B}' | '\u{061E}'..='\u{061F}'
        | '\u{066A}'..='\u{066D}' | '\u{06D4}' => UnicodeCategory::Po,
        _ => UnicodeCategory::Other,
    }
}

/// Result of `encode_info`: whether to encode the inside and outside characters
/// of an attention run.
pub struct EncodeInfoResult {
    pub inside: bool,
    pub outside: bool,
}

/// Check whether to encode (as a character reference) the characters
/// surrounding an attention run.
///
/// Port of JS `lib/util/encode-info.js`.
///
/// In JS, `classifyCharacter` returns:
/// - `undefined` for letters/other
/// - `1` for whitespace
/// - `2` for punctuation
///
/// We map: Other -> None (letter), Whitespace -> Some(1), Punctuation -> Some(2)
pub fn encode_info(outside: Option<char>, inside: Option<char>, marker: char) -> EncodeInfoResult {
    let outside_kind = outside.map(classify_character);
    let inside_kind = inside.map(classify_character);

    // Map CharacterKind to the JS classification:
    // JS undefined (letter) = CharacterKind::Other or None
    // JS 1 (whitespace) = CharacterKind::Whitespace
    // JS 2 (punctuation) = CharacterKind::Punctuation
    fn to_js_kind(kind: Option<CharacterKind>) -> Option<u8> {
        match kind {
            None | Some(CharacterKind::Other) => None, // letter / undefined in JS
            Some(CharacterKind::Whitespace) => Some(1),
            Some(CharacterKind::Punctuation) => Some(2),
        }
    }

    let ok = to_js_kind(outside_kind);
    let ik = to_js_kind(inside_kind);

    match ok {
        // Letter outside
        None => match ik {
            None => {
                // Letter inside
                if marker == '_' {
                    EncodeInfoResult {
                        inside: true,
                        outside: true,
                    }
                } else {
                    EncodeInfoResult {
                        inside: false,
                        outside: false,
                    }
                }
            }
            Some(1) => {
                // Whitespace inside
                EncodeInfoResult {
                    inside: true,
                    outside: true,
                }
            }
            Some(_) => {
                // Punctuation inside
                EncodeInfoResult {
                    inside: false,
                    outside: true,
                }
            }
        },
        // Whitespace outside
        Some(1) => match ik {
            None => EncodeInfoResult {
                inside: false,
                outside: false,
            },
            Some(1) => EncodeInfoResult {
                inside: true,
                outside: true,
            },
            Some(_) => EncodeInfoResult {
                inside: false,
                outside: false,
            },
        },
        // Punctuation outside
        Some(_) => match ik {
            None => EncodeInfoResult {
                inside: false,
                outside: false,
            },
            Some(1) => EncodeInfoResult {
                inside: true,
                outside: false,
            },
            Some(_) => EncodeInfoResult {
                inside: false,
                outside: false,
            },
        },
    }
}

/// Check whether a link can be formatted as an autolink.
///
/// Port of JS `lib/util/format-link-as-autolink.js`.
pub fn format_link_as_autolink(node: &markdown::mdast::Link, state: &State) -> bool {
    let raw = node_to_string(&Node::Link(node.clone()));

    !state.options.resource_link
        && !node.url.is_empty()
        && node.title.is_none()
        && node.children.len() == 1
        && matches!(&node.children[0], Node::Text(_))
        && (raw == node.url || format!("mailto:{}", raw) == node.url)
        && regex::Regex::new(r"(?i)^[a-z][a-z+.-]+:")
            .unwrap()
            .is_match(&node.url)
        && !regex::Regex::new(r"[\x00-\x20<>\x7F]")
            .unwrap()
            .is_match(&node.url)
}

/// Get the list item indent style.
///
/// Port of JS `lib/util/check-list-item-indent.js`.
pub fn check_list_item_indent(state: &State) -> &str {
    let style = state.options.list_item_indent.as_str();
    if style != "tab" && style != "one" && style != "mixed" {
        panic!(
            "Cannot serialize items with `{}` for `options.listItemIndent`, expected `tab`, `one`, or `mixed`",
            style
        );
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest_streak() {
        assert_eq!(longest_streak("abc", '`'), 0);
        assert_eq!(longest_streak("a`b``c```d", '`'), 3);
        assert_eq!(longest_streak("~~~", '~'), 3);
    }

    #[test]
    fn test_node_to_string() {
        let node = Node::Text(markdown::mdast::Text {
            value: "hello".to_string(),
            position: None,
        });
        assert_eq!(node_to_string(&node), "hello");
    }
}
