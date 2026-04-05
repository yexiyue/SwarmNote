use crate::types::{ConstructName, UnsafePattern};

/// List of constructs that occur in phrasing (paragraphs, headings), but cannot
/// contain things like attention (emphasis, strong), images, or links.
/// So they sort of cancel each other out.
fn full_phrasing_spans() -> Vec<ConstructName> {
    vec![
        ConstructName::Autolink,
        ConstructName::DestinationLiteral,
        ConstructName::DestinationRaw,
        ConstructName::Reference,
        ConstructName::TitleQuote,
        ConstructName::TitleApostrophe,
    ]
}

/// Default unsafe patterns.
///
/// Port of JS `lib/unsafe.js`. These define characters that need escaping
/// in certain markdown contexts.
pub fn default_unsafe_patterns() -> Vec<UnsafePattern> {
    let fps = full_phrasing_spans;

    vec![
        // Tab after/before eol in phrasing
        UnsafePattern {
            character: '\t',
            before: None,
            after: Some("[\\r\\n]".to_string()),
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: vec![],
        },
        UnsafePattern {
            character: '\t',
            before: Some("[\\r\\n]".to_string()),
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: vec![],
        },
        // Tab in code fenced lang
        UnsafePattern {
            character: '\t',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![
                ConstructName::CodeFencedLangGraveAccent,
                ConstructName::CodeFencedLangTilde,
            ],
            not_in_construct: vec![],
        },
        // Carriage return in various constructs
        UnsafePattern {
            character: '\r',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![
                ConstructName::CodeFencedLangGraveAccent,
                ConstructName::CodeFencedLangTilde,
                ConstructName::CodeFencedMetaGraveAccent,
                ConstructName::CodeFencedMetaTilde,
                ConstructName::DestinationLiteral,
                ConstructName::HeadingAtx,
            ],
            not_in_construct: vec![],
        },
        // Newline in various constructs
        UnsafePattern {
            character: '\n',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![
                ConstructName::CodeFencedLangGraveAccent,
                ConstructName::CodeFencedLangTilde,
                ConstructName::CodeFencedMetaGraveAccent,
                ConstructName::CodeFencedMetaTilde,
                ConstructName::DestinationLiteral,
                ConstructName::HeadingAtx,
            ],
            not_in_construct: vec![],
        },
        // Space after/before eol in phrasing
        UnsafePattern {
            character: ' ',
            before: None,
            after: Some("[\\r\\n]".to_string()),
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: vec![],
        },
        UnsafePattern {
            character: ' ',
            before: Some("[\\r\\n]".to_string()),
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: vec![],
        },
        // Space in code fenced lang
        UnsafePattern {
            character: ' ',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![
                ConstructName::CodeFencedLangGraveAccent,
                ConstructName::CodeFencedLangTilde,
            ],
            not_in_construct: vec![],
        },
        // Exclamation mark can start an image
        UnsafePattern {
            character: '!',
            before: None,
            after: Some("\\[".to_string()),
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: fps(),
        },
        // Double quote can break out of a title
        UnsafePattern {
            character: '"',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::TitleQuote],
            not_in_construct: vec![],
        },
        // Number sign could start an ATX heading at break
        UnsafePattern {
            character: '#',
            before: None,
            after: None,
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        // Number sign at end of ATX heading
        UnsafePattern {
            character: '#',
            before: None,
            after: Some("(?:[\\r\\n]|$)".to_string()),
            at_break: false,
            in_construct: vec![ConstructName::HeadingAtx],
            not_in_construct: vec![],
        },
        // Ampersand could start a character reference
        UnsafePattern {
            character: '&',
            before: None,
            after: Some("[#A-Za-z]".to_string()),
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: vec![],
        },
        // Apostrophe can break out of a title
        UnsafePattern {
            character: '\'',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::TitleApostrophe],
            not_in_construct: vec![],
        },
        // Left paren could break out of destination raw
        UnsafePattern {
            character: '(',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::DestinationRaw],
            not_in_construct: vec![],
        },
        // Left paren after `]` could make something into a link or image
        UnsafePattern {
            character: '(',
            before: Some("\\]".to_string()),
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: fps(),
        },
        // Right paren could start a list item or break out of destination raw
        UnsafePattern {
            character: ')',
            before: Some("\\d+".to_string()),
            after: None,
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        UnsafePattern {
            character: ')',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::DestinationRaw],
            not_in_construct: vec![],
        },
        // Asterisk can start thematic breaks, list items, emphasis, strong
        UnsafePattern {
            character: '*',
            before: None,
            after: Some("(?:[ \\t\\r\\n*])".to_string()),
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        UnsafePattern {
            character: '*',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: fps(),
        },
        // Plus sign could start a list item
        UnsafePattern {
            character: '+',
            before: None,
            after: Some("(?:[ \\t\\r\\n])".to_string()),
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        // Dash can start thematic breaks, list items, setext heading underlines
        UnsafePattern {
            character: '-',
            before: None,
            after: Some("(?:[ \\t\\r\\n-])".to_string()),
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        // Dot could start a list item
        UnsafePattern {
            character: '.',
            before: Some("\\d+".to_string()),
            after: Some("(?:[ \\t\\r\\n]|$)".to_string()),
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        // Less than can start html or autolink
        UnsafePattern {
            character: '<',
            before: None,
            after: Some("[!/?A-Za-z]".to_string()),
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        UnsafePattern {
            character: '<',
            before: None,
            after: Some("[!/?A-Za-z]".to_string()),
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: fps(),
        },
        UnsafePattern {
            character: '<',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::DestinationLiteral],
            not_in_construct: vec![],
        },
        // Equals sign can start setext heading underlines
        UnsafePattern {
            character: '=',
            before: None,
            after: None,
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        // Greater than can start block quotes
        UnsafePattern {
            character: '>',
            before: None,
            after: None,
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        UnsafePattern {
            character: '>',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::DestinationLiteral],
            not_in_construct: vec![],
        },
        // Left bracket can start definitions, references, labels
        UnsafePattern {
            character: '[',
            before: None,
            after: None,
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        UnsafePattern {
            character: '[',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: fps(),
        },
        UnsafePattern {
            character: '[',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Label, ConstructName::Reference],
            not_in_construct: vec![],
        },
        // Backslash can start an escape or hard break
        UnsafePattern {
            character: '\\',
            before: None,
            after: Some("[\\r\\n]".to_string()),
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: vec![],
        },
        // Right bracket can exit labels
        UnsafePattern {
            character: ']',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Label, ConstructName::Reference],
            not_in_construct: vec![],
        },
        // Underscore can start emphasis, strong, or thematic break
        UnsafePattern {
            character: '_',
            before: None,
            after: None,
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        UnsafePattern {
            character: '_',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: fps(),
        },
        // Grave accent can start code (fenced or text)
        UnsafePattern {
            character: '`',
            before: None,
            after: None,
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
        UnsafePattern {
            character: '`',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![
                ConstructName::CodeFencedLangGraveAccent,
                ConstructName::CodeFencedMetaGraveAccent,
            ],
            not_in_construct: vec![],
        },
        UnsafePattern {
            character: '`',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: fps(),
        },
        // Tilde can start code (fenced)
        UnsafePattern {
            character: '~',
            before: None,
            after: None,
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        },
    ]
}
