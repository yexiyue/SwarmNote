use std::collections::HashMap;

use markdown::mdast::Node;

/// Construct names for context tracking.
///
/// These are semantic labels (similar to micromark events) that track what
/// construct we are currently serializing within.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstructName {
    Autolink,
    Blockquote,
    CodeIndented,
    CodeFenced,
    CodeFencedLangGraveAccent,
    CodeFencedLangTilde,
    CodeFencedMetaGraveAccent,
    CodeFencedMetaTilde,
    Definition,
    DestinationLiteral,
    DestinationRaw,
    Emphasis,
    HeadingAtx,
    HeadingSetext,
    Image,
    ImageReference,
    Label,
    Link,
    LinkReference,
    List,
    ListItem,
    Paragraph,
    Phrasing,
    Reference,
    Strong,
    TitleApostrophe,
    TitleQuote,
    // GFM constructs
    Table,
    TableCell,
    TableRow,
    Strikethrough,
    /// Custom construct name for extensions.
    Custom(String),
}

impl ConstructName {
    /// Convert a construct name string (as used in JS) to the enum variant.
    pub fn parse(s: &str) -> Self {
        match s {
            "autolink" => Self::Autolink,
            "blockquote" => Self::Blockquote,
            "codeIndented" => Self::CodeIndented,
            "codeFenced" => Self::CodeFenced,
            "codeFencedLangGraveAccent" => Self::CodeFencedLangGraveAccent,
            "codeFencedLangTilde" => Self::CodeFencedLangTilde,
            "codeFencedMetaGraveAccent" => Self::CodeFencedMetaGraveAccent,
            "codeFencedMetaTilde" => Self::CodeFencedMetaTilde,
            "definition" => Self::Definition,
            "destinationLiteral" => Self::DestinationLiteral,
            "destinationRaw" => Self::DestinationRaw,
            "emphasis" => Self::Emphasis,
            "headingAtx" => Self::HeadingAtx,
            "headingSetext" => Self::HeadingSetext,
            "image" => Self::Image,
            "imageReference" => Self::ImageReference,
            "label" => Self::Label,
            "link" => Self::Link,
            "linkReference" => Self::LinkReference,
            "list" => Self::List,
            "listItem" => Self::ListItem,
            "paragraph" => Self::Paragraph,
            "phrasing" => Self::Phrasing,
            "reference" => Self::Reference,
            "strong" => Self::Strong,
            "titleApostrophe" => Self::TitleApostrophe,
            "titleQuote" => Self::TitleQuote,
            "table" => Self::Table,
            "tableCell" => Self::TableCell,
            "tableRow" => Self::TableRow,
            "strikethrough" => Self::Strikethrough,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Handler function signature for serializing a node to markdown.
///
/// Arguments: (node, parent, state, info) -> serialized markdown string.
pub type HandlerFn = fn(&Node, Option<&Node>, &mut State, &Info) -> String;

/// Peek function signature - returns the first character a handler would produce.
///
/// Used by container_phrasing to determine the `after` context character.
pub type PeekFn = fn(&Node, Option<&Node>, &mut State, &Info) -> String;

/// Join function signature for determining spacing between flow children.
///
/// Returns:
/// - `Some(n)` where n >= 0: use n blank lines between (0 = flush, 1 = one blank line)
/// - `Some(-1)`: nodes cannot be adjacent, insert a comment break (`<!---->`)
/// - `None`: no opinion, defer to other join functions
pub type JoinFn = fn(&Node, &Node, &Node, &State) -> Option<i32>;

/// Schema that defines when a character cannot occur.
///
/// Mirrors the JS `Unsafe` type from mdast-util-to-markdown.
#[derive(Debug, Clone)]
pub struct UnsafePattern {
    /// Single unsafe character.
    pub character: char,
    /// Regex pattern: `character` is bad when this is before it.
    /// Cannot be used together with `at_break`.
    pub before: Option<String>,
    /// Regex pattern: `character` is bad when this is after it.
    pub after: Option<String>,
    /// `character` is bad at a line break (start of line).
    /// Cannot be used together with `before`.
    pub at_break: bool,
    /// Constructs where this pattern is active.
    pub in_construct: Vec<ConstructName>,
    /// Constructs where this pattern is inactive (overrides `in_construct`).
    pub not_in_construct: Vec<ConstructName>,
}

/// Extension for customizing markdown serialization.
///
/// Merging semantics (matching JS behavior):
/// - `handlers`: last-wins (later registrations override earlier ones)
/// - `unsafe_patterns`: accumulated (all patterns are checked)
/// - `join`: accumulated (executed in order, first non-None wins)
pub struct Extension {
    pub handlers: HashMap<String, HandlerFn>,
    pub unsafe_patterns: Vec<UnsafePattern>,
    pub join: Vec<JoinFn>,
}

/// Configuration options for `to_markdown()`.
///
/// Mirrors the JS `Options` type.
#[derive(Debug, Clone)]
pub struct Options {
    /// Marker for unordered list bullets (default: `'*'`).
    pub bullet: char,
    /// Alternative bullet marker when `bullet` can't be used (default: `'-'`).
    pub bullet_other: Option<char>,
    /// Marker for ordered list bullets (default: `'.'`).
    pub bullet_ordered: char,
    /// Whether to add closing `#` signs to ATX headings (default: `false`).
    pub close_atx: bool,
    /// Marker for emphasis (default: `'*'`).
    pub emphasis: char,
    /// Marker for fenced code (default: `` '`' ``).
    pub fence: char,
    /// Whether to always use fenced code (default: `true`).
    pub fences: bool,
    /// Whether to increment ordered list item counters (default: `true`).
    pub increment_list_marker: bool,
    /// How to indent list item content: `"one"`, `"tab"`, or `"mixed"` (default: `"one"`).
    pub list_item_indent: String,
    /// Marker for titles in links/images/definitions (default: `'"'`).
    pub quote: char,
    /// Whether to always use resource links (default: `false`).
    pub resource_link: bool,
    /// Marker for thematic breaks (default: `'*'`).
    pub rule: char,
    /// Number of markers for thematic breaks (default: `3`).
    pub rule_repetition: usize,
    /// Whether to add spaces between thematic break markers (default: `false`).
    pub rule_spaces: bool,
    /// Whether to use setext headings when possible (default: `false`).
    pub setext: bool,
    /// Marker for strong (default: `'*'`).
    pub strong: char,
    /// Whether to join definitions without a blank line (default: `false`).
    pub tight_definitions: bool,
    /// Whether to include GFM extensions (tables, strikethrough, task lists) by default.
    pub gfm: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            bullet: '*',
            bullet_other: None,
            bullet_ordered: '.',
            close_atx: false,
            emphasis: '*',
            fence: '`',
            fences: true,
            increment_list_marker: true,
            list_item_indent: "one".to_string(),
            quote: '"',
            resource_link: false,
            rule: '*',
            rule_repetition: 3,
            rule_spaces: false,
            setext: false,
            strong: '*',
            tight_definitions: false,
            gfm: true,
        }
    }
}

/// Info on the surrounding of the node being serialized.
///
/// Combines `SafeFields` and `TrackFields` from the JS version.
#[derive(Debug, Clone)]
pub struct Info {
    /// Characters before this (guaranteed to be one, can be more).
    pub before: String,
    /// Characters after this (guaranteed to be one, can be more).
    pub after: String,
    /// Current line number (1-indexed).
    pub line: usize,
    /// Current column number (1-indexed).
    pub column: usize,
    /// Number of columns each line will be shifted by wrapping nodes.
    pub line_shift: usize,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            before: String::new(),
            after: String::new(),
            line: 1,
            column: 1,
            line_shift: 0,
        }
    }
}

/// Configuration for the `safe()` function.
pub struct SafeConfig {
    /// Characters before this.
    pub before: String,
    /// Characters after this.
    pub after: String,
    /// Extra characters that must be encoded as character references
    /// instead of escaped with backslash.
    pub encode: Vec<char>,
}

/// Whether to encode things around attention markers.
#[derive(Debug, Clone, Default)]
pub struct EncodeSurrounding {
    /// Whether to encode before.
    pub before: bool,
    /// Whether to encode after.
    pub after: bool,
}

/// Positional tracking fields.
#[derive(Debug, Clone)]
pub struct TrackFields {
    /// Current line (1-indexed).
    pub line: usize,
    /// Current column (1-indexed).
    pub column: usize,
    /// Number of columns each line will be shifted.
    pub line_shift: usize,
}

impl Default for TrackFields {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            line_shift: 0,
        }
    }
}

use crate::state::State;
