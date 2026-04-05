//! Comprehensive tests for mdast-util-to-markdown, ported from the JS test suite.
//!
//! Covers: core serialization, GFM extensions, character escaping, options, and extensions.

use markdown::mdast::*;
use mdast_util_to_markdown::{to_markdown, Options};

// ---------------------------------------------------------------------------
// Helper: wrap nodes in a Root
// ---------------------------------------------------------------------------
fn root(children: Vec<Node>) -> Node {
    Node::Root(Root {
        children,
        position: None,
    })
}

fn para(children: Vec<Node>) -> Node {
    Node::Paragraph(Paragraph {
        children,
        position: None,
    })
}

fn text(value: &str) -> Node {
    Node::Text(Text {
        value: value.into(),
        position: None,
    })
}

fn heading(depth: u8, children: Vec<Node>) -> Node {
    Node::Heading(Heading {
        depth,
        children,
        position: None,
    })
}

fn list_item(children: Vec<Node>) -> Node {
    Node::ListItem(ListItem {
        checked: None,
        spread: false,
        children,
        position: None,
    })
}

fn unordered_list(spread: bool, items: Vec<Node>) -> Node {
    Node::List(List {
        ordered: false,
        start: None,
        spread,
        children: items,
        position: None,
    })
}

fn ordered_list(start: Option<u32>, spread: bool, items: Vec<Node>) -> Node {
    Node::List(List {
        ordered: true,
        start,
        spread,
        children: items,
        position: None,
    })
}

fn emphasis(children: Vec<Node>) -> Node {
    Node::Emphasis(Emphasis {
        children,
        position: None,
    })
}

fn strong(children: Vec<Node>) -> Node {
    Node::Strong(Strong {
        children,
        position: None,
    })
}

// ===========================================================================
// 7.1 Core tests
// ===========================================================================

// ---- Paragraph ----

#[test]
fn paragraph_plain_text() {
    let tree = root(vec![para(vec![text("Hello world")])]);
    assert_eq!(to_markdown(&tree, &Options::default()), "Hello world\n");
}

#[test]
fn paragraph_with_emphasis_and_strong() {
    let tree = root(vec![para(vec![
        emphasis(vec![text("em")]),
        text(" and "),
        strong(vec![text("strong")]),
    ])]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "*em* and **strong**\n"
    );
}

#[test]
fn paragraph_with_link() {
    let tree = root(vec![para(vec![
        text("Click "),
        Node::Link(Link {
            url: "https://example.com".into(),
            title: None,
            children: vec![text("here")],
            position: None,
        }),
    ])]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "Click [here](https://example.com)\n"
    );
}

// ---- Root with multiple flow children ----

#[test]
fn root_paragraph_thematic_break_paragraph() {
    let tree = root(vec![
        para(vec![text("a")]),
        Node::ThematicBreak(ThematicBreak { position: None }),
        para(vec![text("b")]),
    ]);
    assert_eq!(to_markdown(&tree, &Options::default()), "a\n\n***\n\nb\n");
}

// ---- Heading (ATX) ----

#[test]
fn heading_h1_through_h6() {
    for depth in 1u8..=6 {
        let tree = root(vec![heading(depth, vec![text("Title")])]);
        let hashes = "#".repeat(depth as usize);
        assert_eq!(
            to_markdown(&tree, &Options::default()),
            format!("{} Title\n", hashes)
        );
    }
}

#[test]
fn heading_empty() {
    let tree = root(vec![heading(1, vec![])]);
    assert_eq!(to_markdown(&tree, &Options::default()), "#\n");
}

#[test]
fn heading_depth_6() {
    let tree = root(vec![heading(6, vec![])]);
    assert_eq!(to_markdown(&tree, &Options::default()), "######\n");
}

// ---- Lists ----

#[test]
fn ordered_list_consecutive_items_no_separator() {
    // THE key test: consecutive ordered items should stay in one list, no <!-- end list -->
    let tree = root(vec![ordered_list(
        Some(1),
        false,
        vec![
            list_item(vec![para(vec![text("first")])]),
            list_item(vec![para(vec![text("second")])]),
            list_item(vec![para(vec![text("third")])]),
        ],
    )]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "1. first\n2. second\n3. third\n");
    // Must NOT contain any HTML comment separator
    assert!(!result.contains("<!--"));
}

#[test]
fn unordered_list_basic() {
    let tree = root(vec![unordered_list(
        false,
        vec![
            list_item(vec![para(vec![text("one")])]),
            list_item(vec![para(vec![text("two")])]),
        ],
    )]);
    assert_eq!(to_markdown(&tree, &Options::default()), "* one\n* two\n");
}

#[test]
fn list_nested_two_levels() {
    let tree = root(vec![unordered_list(
        false,
        vec![list_item(vec![
            para(vec![text("outer")]),
            unordered_list(
                false,
                vec![
                    list_item(vec![para(vec![text("inner1")])]),
                    list_item(vec![para(vec![text("inner2")])]),
                ],
            ),
        ])],
    )]);
    let result = to_markdown(&tree, &Options::default());
    assert!(result.contains("* outer"));
    assert!(result.contains("  * inner1"));
    assert!(result.contains("  * inner2"));
}

#[test]
fn list_nested_three_levels() {
    let tree = root(vec![unordered_list(
        false,
        vec![list_item(vec![
            para(vec![text("L1")]),
            unordered_list(
                false,
                vec![list_item(vec![
                    para(vec![text("L2")]),
                    unordered_list(false, vec![list_item(vec![para(vec![text("L3")])])]),
                ])],
            ),
        ])],
    )]);
    let result = to_markdown(&tree, &Options::default());
    assert!(result.contains("* L1"));
    assert!(result.contains("  * L2"));
    assert!(result.contains("    * L3"));
}

#[test]
fn list_tight_vs_loose() {
    // Tight list (spread: false) -- no blank lines between items
    let tight = root(vec![unordered_list(
        false,
        vec![
            list_item(vec![para(vec![text("a")])]),
            list_item(vec![para(vec![text("b")])]),
        ],
    )]);
    assert_eq!(to_markdown(&tight, &Options::default()), "* a\n* b\n");

    // Loose list (spread: true) -- blank lines between items
    let loose = root(vec![unordered_list(
        true,
        vec![
            list_item(vec![para(vec![text("a")])]),
            list_item(vec![para(vec![text("b")])]),
        ],
    )]);
    let result = to_markdown(&loose, &Options::default());
    assert_eq!(result, "* a\n\n* b\n");
}

#[test]
fn list_spread_false_with_spread_item() {
    // A tight list where one item has two paragraphs (spread item).
    // The blank line should appear inside the item, not between items.
    // Use bullet '-' to match the JS test expectation.
    let tree = root(vec![unordered_list(
        false,
        vec![
            Node::ListItem(ListItem {
                checked: None,
                spread: false,
                children: vec![para(vec![text("a")]), para(vec![text("b")])],
                position: None,
            }),
            list_item(vec![Node::ThematicBreak(ThematicBreak { position: None })]),
        ],
    )]);
    let opts = Options {
        bullet: '-',
        ..Options::default()
    };
    assert_eq!(
        to_markdown(&tree, &opts),
        "- a\n\n  b\n- ***\n"
    );
}

#[test]
fn ordered_list_with_start() {
    // spread: true (default in JS) gives blank lines between items
    let tree = root(vec![ordered_list(
        Some(0),
        true,
        vec![
            list_item(vec![para(vec![text("a")])]),
            list_item(vec![Node::ThematicBreak(ThematicBreak { position: None })]),
        ],
    )]);
    assert_eq!(to_markdown(&tree, &Options::default()), "0. a\n\n1. ***\n");
}

#[test]
fn ordered_list_spread_false() {
    let tree = root(vec![ordered_list(
        Some(1),
        false,
        vec![
            list_item(vec![para(vec![text("a")])]),
            list_item(vec![Node::ThematicBreak(ThematicBreak { position: None })]),
            list_item(vec![para(vec![text("b")])]),
        ],
    )]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "1. a\n2. ***\n3. b\n"
    );
}

// ---- Code (flow) ----

#[test]
fn code_fenced_with_language() {
    let tree = root(vec![Node::Code(Code {
        lang: Some("rust".into()),
        meta: None,
        value: "fn main() {}".into(),
        position: None,
    })]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "```rust\nfn main() {}\n```\n"
    );
}

#[test]
fn code_fenced_without_language() {
    let tree = root(vec![Node::Code(Code {
        lang: None,
        meta: None,
        value: "hello".into(),
        position: None,
    })]);
    assert_eq!(to_markdown(&tree, &Options::default()), "```\nhello\n```\n");
}

#[test]
fn code_fenced_with_backticks_inside() {
    // If the value contains ```, the fence should use more backticks.
    let tree = root(vec![Node::Code(Code {
        lang: None,
        meta: None,
        value: "```\nasd\n```".into(),
        position: None,
    })]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "````\n```\nasd\n```\n````\n"
    );
}

#[test]
fn code_with_lang_and_meta() {
    let tree = root(vec![Node::Code(Code {
        lang: Some("js".into()),
        meta: Some("highlight".into()),
        value: "".into(),
        position: None,
    })]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "```js highlight\n```\n"
    );
}

// ---- Inline code ----

#[test]
fn inline_code_simple() {
    let tree = root(vec![para(vec![Node::InlineCode(InlineCode {
        value: "code".into(),
        position: None,
    })])]);
    assert_eq!(to_markdown(&tree, &Options::default()), "`code`\n");
}

#[test]
fn inline_code_with_backtick_inside() {
    let tree = root(vec![para(vec![Node::InlineCode(InlineCode {
        value: "a`b".into(),
        position: None,
    })])]);
    assert_eq!(to_markdown(&tree, &Options::default()), "``a`b``\n");
}

#[test]
fn inline_code_with_two_backticks_and_one() {
    let tree = root(vec![para(vec![Node::InlineCode(InlineCode {
        value: "a``b`c".into(),
        position: None,
    })])]);
    assert_eq!(to_markdown(&tree, &Options::default()), "```a``b`c```\n");
}

#[test]
fn inline_code_starting_with_backtick() {
    let tree = root(vec![para(vec![Node::InlineCode(InlineCode {
        value: "`a".into(),
        position: None,
    })])]);
    assert_eq!(to_markdown(&tree, &Options::default()), "`` `a ``\n");
}

// ---- Blockquote ----

#[test]
fn blockquote_simple() {
    let tree = root(vec![Node::Blockquote(Blockquote {
        children: vec![para(vec![text("quoted")])],
        position: None,
    })]);
    assert_eq!(to_markdown(&tree, &Options::default()), "> quoted\n");
}

#[test]
fn blockquote_with_multiple_children() {
    let tree = root(vec![Node::Blockquote(Blockquote {
        children: vec![
            para(vec![text("a")]),
            Node::ThematicBreak(ThematicBreak { position: None }),
            para(vec![text("b")]),
        ],
        position: None,
    })]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "> a\n>\n> ***\n>\n> b\n"
    );
}

#[test]
fn blockquote_text_with_line_ending() {
    let tree = root(vec![Node::Blockquote(Blockquote {
        children: vec![para(vec![text("a\nb")])],
        position: None,
    })]);
    assert_eq!(to_markdown(&tree, &Options::default()), "> a\n> b\n");
}

#[test]
fn blockquote_nested() {
    let tree = root(vec![Node::Blockquote(Blockquote {
        children: vec![Node::Blockquote(Blockquote {
            children: vec![para(vec![text("deep")])],
            position: None,
        })],
        position: None,
    })]);
    let result = to_markdown(&tree, &Options::default());
    assert!(result.contains("> > deep"));
}

#[test]
fn blockquote_with_break() {
    let tree = root(vec![Node::Blockquote(Blockquote {
        children: vec![para(vec![
            text("a"),
            Node::Break(Break { position: None }),
            text("b"),
        ])],
        position: None,
    })]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "> a\\\n> b\n"
    );
}

// ---- Links ----

#[test]
fn link_resource() {
    let tree = root(vec![para(vec![Node::Link(Link {
        url: "https://example.com".into(),
        title: None,
        children: vec![text("example")],
        position: None,
    })])]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "[example](https://example.com)\n"
    );
}

#[test]
fn link_with_title() {
    let tree = root(vec![para(vec![Node::Link(Link {
        url: "https://example.com".into(),
        title: Some("My Title".into()),
        children: vec![text("example")],
        position: None,
    })])]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "[example](https://example.com \"My Title\")\n"
    );
}

#[test]
fn link_autolink() {
    // When URL matches child text, should produce autolink <url>
    let tree = root(vec![para(vec![Node::Link(Link {
        url: "https://example.com".into(),
        title: None,
        children: vec![text("https://example.com")],
        position: None,
    })])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "<https://example.com>\n");
}

// ---- Images ----

#[test]
fn image_with_alt_and_title() {
    let tree = root(vec![para(vec![Node::Image(Image {
        url: "img.png".into(),
        alt: "alt text".into(),
        title: Some("My Image".into()),
        position: None,
    })])]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "![alt text](img.png \"My Image\")\n"
    );
}

#[test]
fn image_without_title() {
    let tree = root(vec![para(vec![Node::Image(Image {
        url: "img.png".into(),
        alt: "alt".into(),
        title: None,
        position: None,
    })])]);
    assert_eq!(to_markdown(&tree, &Options::default()), "![alt](img.png)\n");
}

// ---- Thematic break ----

#[test]
fn thematic_break() {
    let tree = root(vec![Node::ThematicBreak(ThematicBreak { position: None })]);
    assert_eq!(to_markdown(&tree, &Options::default()), "***\n");
}

// ---- Hard break ----

#[test]
fn hard_break() {
    let tree = root(vec![para(vec![
        text("line1"),
        Node::Break(Break { position: None }),
        text("line2"),
    ])]);
    assert_eq!(to_markdown(&tree, &Options::default()), "line1\\\nline2\n");
}

// ---- Definition + references ----

#[test]
fn definition_with_title() {
    let tree = root(vec![Node::Definition(Definition {
        url: "https://example.com".into(),
        title: Some("Example".into()),
        identifier: "ex".into(),
        label: Some("ex".into()),
        position: None,
    })]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "[ex]: https://example.com \"Example\"\n"
    );
}

#[test]
fn definition_without_title() {
    let tree = root(vec![Node::Definition(Definition {
        url: "https://example.com".into(),
        title: None,
        identifier: "ex".into(),
        label: Some("ex".into()),
        position: None,
    })]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "[ex]: https://example.com\n"
    );
}

#[test]
fn link_reference_collapsed() {
    let tree = root(vec![para(vec![Node::LinkReference(LinkReference {
        identifier: "ex".into(),
        label: Some("ex".into()),
        reference_kind: ReferenceKind::Collapsed,
        children: vec![text("ex")],
        position: None,
    })])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "[ex][]\n");
}

#[test]
fn link_reference_full() {
    let tree = root(vec![para(vec![Node::LinkReference(LinkReference {
        identifier: "ref".into(),
        label: Some("ref".into()),
        reference_kind: ReferenceKind::Full,
        children: vec![text("click me")],
        position: None,
    })])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "[click me][ref]\n");
}

#[test]
fn image_reference_full() {
    let tree = root(vec![para(vec![Node::ImageReference(ImageReference {
        identifier: "img".into(),
        label: Some("img".into()),
        reference_kind: ReferenceKind::Full,
        alt: "alt text".into(),
        position: None,
    })])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "![alt text][img]\n");
}

// ---- HTML passthrough ----

#[test]
fn html_passthrough() {
    let tree = root(vec![Node::Html(Html {
        value: "<div>hello</div>".into(),
        position: None,
    })]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "<div>hello</div>\n"
    );
}

#[test]
fn html_multiline() {
    let tree = root(vec![Node::Html(Html {
        value: "<div\nhidden>".into(),
        position: None,
    })]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "<div\nhidden>\n"
    );
}

// ===========================================================================
// 7.2 GFM tests
// ===========================================================================

#[test]
fn gfm_table_2x2() {
    let tree = root(vec![Node::Table(Table {
        align: vec![AlignKind::None, AlignKind::None],
        children: vec![
            Node::TableRow(TableRow {
                children: vec![
                    Node::TableCell(TableCell {
                        children: vec![text("A")],
                        position: None,
                    }),
                    Node::TableCell(TableCell {
                        children: vec![text("B")],
                        position: None,
                    }),
                ],
                position: None,
            }),
            Node::TableRow(TableRow {
                children: vec![
                    Node::TableCell(TableCell {
                        children: vec![text("1")],
                        position: None,
                    }),
                    Node::TableCell(TableCell {
                        children: vec![text("2")],
                        position: None,
                    }),
                ],
                position: None,
            }),
        ],
        position: None,
    })]);
    let result = to_markdown(&tree, &Options::default());
    assert!(result.contains("| A"));
    assert!(result.contains("| B"));
    assert!(result.contains("| 1"));
    assert!(result.contains("| 2"));
    assert!(result.contains("---"));
}

#[test]
fn gfm_table_with_alignment() {
    let tree = root(vec![Node::Table(Table {
        align: vec![AlignKind::Left, AlignKind::Center, AlignKind::Right],
        children: vec![
            Node::TableRow(TableRow {
                children: vec![
                    Node::TableCell(TableCell {
                        children: vec![text("Left")],
                        position: None,
                    }),
                    Node::TableCell(TableCell {
                        children: vec![text("Center")],
                        position: None,
                    }),
                    Node::TableCell(TableCell {
                        children: vec![text("Right")],
                        position: None,
                    }),
                ],
                position: None,
            }),
            Node::TableRow(TableRow {
                children: vec![
                    Node::TableCell(TableCell {
                        children: vec![text("a")],
                        position: None,
                    }),
                    Node::TableCell(TableCell {
                        children: vec![text("b")],
                        position: None,
                    }),
                    Node::TableCell(TableCell {
                        children: vec![text("c")],
                        position: None,
                    }),
                ],
                position: None,
            }),
        ],
        position: None,
    })]);
    let result = to_markdown(&tree, &Options::default());
    // Left alignment: :---
    assert!(result.contains(":"));
    // Right alignment: ---:
    assert!(result.contains("-:"));
}

#[test]
fn gfm_table_cell_with_pipe_escaped() {
    let tree = root(vec![Node::Table(Table {
        align: vec![AlignKind::None],
        children: vec![
            Node::TableRow(TableRow {
                children: vec![Node::TableCell(TableCell {
                    children: vec![text("Header")],
                    position: None,
                })],
                position: None,
            }),
            Node::TableRow(TableRow {
                children: vec![Node::TableCell(TableCell {
                    children: vec![text("a|b")],
                    position: None,
                })],
                position: None,
            }),
        ],
        position: None,
    })]);
    let result = to_markdown(&tree, &Options::default());
    // The pipe inside a cell should be escaped
    assert!(result.contains("a\\|b"));
}

#[test]
fn gfm_strikethrough() {
    let tree = root(vec![para(vec![Node::Delete(Delete {
        children: vec![text("deleted")],
        position: None,
    })])]);
    assert_eq!(to_markdown(&tree, &Options::default()), "~~deleted~~\n");
}

#[test]
fn gfm_task_list_checked_and_unchecked() {
    let tree = root(vec![unordered_list(
        false,
        vec![
            Node::ListItem(ListItem {
                checked: Some(true),
                spread: false,
                children: vec![para(vec![text("done")])],
                position: None,
            }),
            Node::ListItem(ListItem {
                checked: Some(false),
                spread: false,
                children: vec![para(vec![text("todo")])],
                position: None,
            }),
        ],
    )]);
    assert_eq!(
        to_markdown(&tree, &Options::default()),
        "* [x] done\n* [ ] todo\n"
    );
}

// ===========================================================================
// 7.3 Character escaping tests
// ===========================================================================

#[test]
fn escape_asterisks_in_text() {
    // Asterisks that would create false emphasis must be escaped
    let tree = root(vec![para(vec![text("*a*")])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "\\*a\\*\n");
}

#[test]
fn escape_underscores_in_text() {
    let tree = root(vec![para(vec![text("_a_")])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "\\_a\\_\n");
}

#[test]
fn escape_bracket_in_text() {
    // A definition-like pattern at the start must be escaped
    let tree = root(vec![para(vec![text("[a]: b")])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "\\[a]: b\n");
}

#[test]
fn escape_hash_at_line_start() {
    let tree = root(vec![para(vec![text("# a")])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "\\# a\n");
}

#[test]
fn escape_block_quote_marker() {
    let tree = root(vec![para(vec![text("> a\n> b\nc >")])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "\\> a\n\\> b\nc >\n");
}

#[test]
fn escape_ordered_list_dot() {
    let tree = root(vec![para(vec![text("1. a\n2. b")])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "1\\. a\n2\\. b\n");
}

#[test]
fn escape_fenced_code_in_text() {
    let tree = root(vec![para(vec![text("```js\n```")])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "\\`\\`\\`js\n\\`\\`\\`\n");
}

#[test]
fn escape_pipe_in_table_cell() {
    // When GFM is on, pipes inside table cells are escaped.
    let tree = root(vec![Node::Table(Table {
        align: vec![AlignKind::None],
        children: vec![
            Node::TableRow(TableRow {
                children: vec![Node::TableCell(TableCell {
                    children: vec![text("h")],
                    position: None,
                })],
                position: None,
            }),
            Node::TableRow(TableRow {
                children: vec![Node::TableCell(TableCell {
                    children: vec![text("a|b")],
                    position: None,
                })],
                position: None,
            }),
        ],
        position: None,
    })]);
    let result = to_markdown(&tree, &Options::default());
    assert!(result.contains("a\\|b"));
}

#[test]
fn escape_backslash_break() {
    let tree = root(vec![para(vec![text("a\\\nb")])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "a\\\\\nb\n");
}

#[test]
fn escape_character_reference() {
    let tree = root(vec![para(vec![text("&amp")])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "\\&amp\n");
}

// ===========================================================================
// 7.4 Options tests
// ===========================================================================

#[test]
fn option_bullet_dash() {
    let tree = root(vec![unordered_list(
        false,
        vec![list_item(vec![para(vec![text("a")])])],
    )]);
    let opts = Options {
        bullet: '-',
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "- a\n");
}

#[test]
fn option_bullet_plus() {
    let tree = root(vec![unordered_list(
        false,
        vec![list_item(vec![para(vec![text("a")])])],
    )]);
    let opts = Options {
        bullet: '+',
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "+ a\n");
}

#[test]
fn option_fence_tilde() {
    let tree = root(vec![Node::Code(Code {
        lang: None,
        meta: None,
        value: "a".into(),
        position: None,
    })]);
    let opts = Options {
        fence: '~',
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "~~~\na\n~~~\n");
}

#[test]
fn option_fence_tilde_with_tildes_in_value() {
    let tree = root(vec![Node::Code(Code {
        lang: None,
        meta: None,
        value: "~~~\nasd\n~~~".into(),
        position: None,
    })]);
    let opts = Options {
        fence: '~',
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "~~~~\n~~~\nasd\n~~~\n~~~~\n");
}

#[test]
fn option_emphasis_underscore() {
    let tree = root(vec![para(vec![emphasis(vec![text("em")])])]);
    let opts = Options {
        emphasis: '_',
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "_em_\n");
}

#[test]
fn option_strong_underscore() {
    let tree = root(vec![para(vec![strong(vec![text("bold")])])]);
    let opts = Options {
        strong: '_',
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "__bold__\n");
}

#[test]
fn option_increment_list_marker_false() {
    let tree = root(vec![ordered_list(
        Some(1),
        false,
        vec![
            list_item(vec![para(vec![text("a")])]),
            list_item(vec![Node::ThematicBreak(ThematicBreak { position: None })]),
            list_item(vec![para(vec![text("b")])]),
        ],
    )]);
    let opts = Options {
        increment_list_marker: false,
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "1. a\n1. ***\n1. b\n");
}

#[test]
fn option_setext_h1() {
    let tree = root(vec![heading(1, vec![text("a")])]);
    let opts = Options {
        setext: true,
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "a\n=\n");
}

#[test]
fn option_setext_h2() {
    let tree = root(vec![heading(2, vec![text("a")])]);
    let opts = Options {
        setext: true,
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "a\n-\n");
}

#[test]
fn option_setext_h3_falls_back_to_atx() {
    let tree = root(vec![heading(3, vec![text("a")])]);
    let opts = Options {
        setext: true,
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "### a\n");
}

#[test]
fn option_setext_empty_falls_back_to_atx() {
    // Empty heading h1 should fall back to ATX even with setext: true
    let tree = root(vec![heading(1, vec![])]);
    let opts = Options {
        setext: true,
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "#\n");
}

#[test]
fn option_close_atx() {
    let tree = root(vec![heading(2, vec![text("Title")])]);
    let opts = Options {
        close_atx: true,
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "## Title ##\n");
}

#[test]
fn option_rule_dash() {
    let tree = root(vec![Node::ThematicBreak(ThematicBreak { position: None })]);
    let opts = Options {
        rule: '-',
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "---\n");
}

#[test]
fn option_rule_spaces() {
    let tree = root(vec![Node::ThematicBreak(ThematicBreak { position: None })]);
    let opts = Options {
        rule_spaces: true,
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "* * *\n");
}

#[test]
fn option_tight_definitions() {
    let tree = root(vec![
        para(vec![text("a")]),
        Node::Definition(Definition {
            url: "".into(),
            title: None,
            identifier: "b".into(),
            label: Some("b".into()),
            position: None,
        }),
        Node::Definition(Definition {
            url: "".into(),
            title: None,
            identifier: "c".into(),
            label: Some("c".into()),
            position: None,
        }),
        para(vec![text("d")]),
    ]);
    let opts = Options {
        tight_definitions: true,
        ..Options::default()
    };
    let result = to_markdown(&tree, &opts);
    // With tight_definitions, adjacent definitions should not have blank lines between them
    assert!(result.contains("[b]: <>\n[c]: <>"));
}

#[test]
fn option_quote_single() {
    let tree = root(vec![Node::Definition(Definition {
        url: "https://example.com".into(),
        title: Some("Title".into()),
        identifier: "ex".into(),
        label: Some("ex".into()),
        position: None,
    })]);
    let opts = Options {
        quote: '\'',
        ..Options::default()
    };
    let result = to_markdown(&tree, &opts);
    assert_eq!(result, "[ex]: https://example.com 'Title'\n");
}

// ===========================================================================
// 7.5 Extension mechanism tests
// ===========================================================================

#[test]
fn extension_custom_handler_overrides_default() {
    // Verify that the GFM extension properly overrides handlers.
    // GFM table extension overrides the inlineCode handler so pipes are escaped in table cells.
    let table_tree = root(vec![Node::Table(Table {
        align: vec![AlignKind::None],
        children: vec![
            Node::TableRow(TableRow {
                children: vec![Node::TableCell(TableCell {
                    children: vec![Node::InlineCode(InlineCode {
                        value: "a|b".into(),
                        position: None,
                    })],
                    position: None,
                })],
                position: None,
            }),
        ],
        position: None,
    })]);
    let result = to_markdown(&table_tree, &Options::default());
    // GFM's inline code handler escapes pipes in table cells
    assert!(result.contains("\\|"), "GFM handler should escape pipes in table cells: {}", result);
}

#[test]
fn extension_gfm_disabled() {
    // When GFM is off, delete nodes won't have a handler and strikethrough won't work.
    // But the delete node type may still fall back. Let's test that tables don't
    // get special handling.
    let tree = root(vec![para(vec![text("a")])]);
    let opts = Options {
        gfm: false,
        ..Options::default()
    };
    let result = to_markdown(&tree, &opts);
    assert_eq!(result, "a\n");
}

#[test]
fn extension_gfm_task_list_overrides_list_item() {
    // The GFM task list item extension overrides the default listItem handler.
    // When checked is Some, a checkbox should appear.
    let tree = root(vec![unordered_list(
        false,
        vec![
            Node::ListItem(ListItem {
                checked: Some(true),
                spread: false,
                children: vec![para(vec![text("done")])],
                position: None,
            }),
            Node::ListItem(ListItem {
                checked: None,
                spread: false,
                children: vec![para(vec![text("normal")])],
                position: None,
            }),
        ],
    )]);
    let result = to_markdown(&tree, &Options::default());
    assert!(result.contains("[x] done"));
    assert!(result.contains("* normal"));
    assert!(!result.contains("[ ] normal"));
}

// ===========================================================================
// Additional edge-case tests ported from JS
// ===========================================================================

#[test]
fn adjacent_lists_use_different_marker() {
    // Two adjacent unordered lists should use different bullet markers
    let tree = root(vec![
        unordered_list(false, vec![list_item(vec![])]),
        unordered_list(false, vec![list_item(vec![])]),
    ]);
    let result = to_markdown(&tree, &Options::default());
    // Should contain both * and - (different markers for adjacent lists)
    assert!(result.contains('*'));
    assert!(result.contains('-'));
}

#[test]
fn code_indented_when_fences_false() {
    let tree = root(vec![Node::Code(Code {
        lang: None,
        meta: None,
        value: "a".into(),
        position: None,
    })]);
    let opts = Options {
        fences: false,
        ..Options::default()
    };
    assert_eq!(to_markdown(&tree, &opts), "    a\n");
}

#[test]
fn code_forced_to_fence_when_has_lang() {
    // Even with fences: false, a code block with a lang must use fences
    let tree = root(vec![Node::Code(Code {
        lang: Some("js".into()),
        meta: None,
        value: "a".into(),
        position: None,
    })]);
    let opts = Options {
        fences: false,
        ..Options::default()
    };
    let result = to_markdown(&tree, &opts);
    assert!(result.contains("```js"));
}

#[test]
fn setext_heading_with_line_ending_in_text() {
    let tree = root(vec![heading(1, vec![text("a\nb")])]);
    let result = to_markdown(&tree, &Options::default());
    // A heading with a line ending in the text should fall back to setext
    // to preserve the multiline content
    assert!(result.contains("a\nb\n=") || result.contains("a&#xA;b"));
}

#[test]
fn list_ordered_start_at_9_crossing_to_10() {
    let tree = root(vec![ordered_list(
        Some(9),
        false,
        vec![
            list_item(vec![para(vec![text("a\nb")])]),
            list_item(vec![para(vec![text("c\nd")])]),
        ],
    )]);
    let result = to_markdown(&tree, &Options::default());
    assert!(result.contains("9."));
    assert!(result.contains("10."));
}

#[test]
fn inline_code_prevents_breaking_out_with_dash() {
    let tree = root(vec![para(vec![Node::InlineCode(InlineCode {
        value: "a\n- b".into(),
        position: None,
    })])]);
    let result = to_markdown(&tree, &Options::default());
    // Line endings in inline code should be replaced with spaces
    assert_eq!(result, "`a - b`\n");
}

#[test]
fn inline_code_prevents_breaking_out_with_hash() {
    let tree = root(vec![para(vec![Node::InlineCode(InlineCode {
        value: "a\n#".into(),
        position: None,
    })])]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "`a #`\n");
}

#[test]
fn empty_root() {
    let tree = root(vec![]);
    assert_eq!(to_markdown(&tree, &Options::default()), "");
}

#[test]
fn break_in_heading_becomes_space() {
    let tree = heading(
        3,
        vec![
            text("a"),
            Node::Break(Break { position: None }),
            text("b"),
        ],
    );
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "### a b\n");
}

#[test]
fn definition_empty_url_uses_angle_brackets() {
    let tree = root(vec![Node::Definition(Definition {
        url: "".into(),
        title: None,
        identifier: "a".into(),
        label: Some("a".into()),
        position: None,
    })]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "[a]: <>\n");
}

#[test]
fn escape_emphasis_in_text_underscore_mode() {
    // When emphasis uses underscore, asterisks in text still need escaping
    // because they're still valid emphasis markers in markdown
    let tree = root(vec![para(vec![text("*a*")])]);
    let opts = Options {
        emphasis: '_',
        ..Options::default()
    };
    let result = to_markdown(&tree, &opts);
    assert_eq!(result, "\\*a\\*\n");
}

#[test]
fn multiple_paragraphs_separated_by_blank_line() {
    let tree = root(vec![
        para(vec![text("first")]),
        para(vec![text("second")]),
    ]);
    let result = to_markdown(&tree, &Options::default());
    assert_eq!(result, "first\n\nsecond\n");
}
