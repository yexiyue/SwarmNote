//! # mdast-util-to-markdown
//!
//! A Rust port of the JS library `mdast-util-to-markdown` (v2.1.2).
//!
//! Turns an mdast syntax tree into markdown. Uses the `markdown` crate's
//! `mdast::Node` type for the AST representation.
//!
//! ## Usage
//!
//! ```rust
//! use markdown::mdast::Node;
//! use mdast_util_to_markdown::{to_markdown, Options};
//!
//! // Create a simple tree
//! let tree = Node::Root(markdown::mdast::Root {
//!     children: vec![],
//!     position: None,
//! });
//!
//! let result = to_markdown(&tree, &Options::default());
//! assert_eq!(result, "");
//! ```

pub mod configure;
pub mod gfm;
pub mod handle;
pub mod join;
pub mod state;
pub mod types;
pub mod unsafe_patterns;
pub mod util;

use markdown::mdast::Node;

pub use gfm::{gfm, gfm_strikethrough, gfm_table, gfm_task_list_item, GfmTableOptions};
pub use types::{
    ConstructName, EncodeSurrounding, Extension, HandlerFn, Info, JoinFn, Options, PeekFn,
    SafeConfig, TrackFields, UnsafePattern,
};

use state::State;

/// Turn an mdast syntax tree into markdown.
///
/// Port of JS `toMarkdown()` from `lib/index.js`.
///
/// # Arguments
///
/// * `tree` - The mdast syntax tree to serialize.
/// * `options` - Configuration options.
///
/// # Returns
///
/// Serialized markdown representing the tree.
pub fn to_markdown(tree: &Node, options: &Options) -> String {
    let mut state = State::new();

    // Set up default handlers.
    state.handlers = handle::default_handlers();

    // Set up default peek handlers.
    state.peek_handlers = handle::default_peek_handlers();

    // Set up default join functions.
    state.join = join::default_join();

    // Set up default unsafe patterns.
    state.unsafe_patterns = unsafe_patterns::default_unsafe_patterns();

    // Apply options.
    state.options = options.clone();

    // Apply GFM extensions by default.
    if options.gfm {
        for ext in gfm::gfm() {
            // Merge handlers
            state.handlers.extend(ext.handlers);
            // Merge unsafe patterns
            state.unsafe_patterns.extend(ext.unsafe_patterns);
            // Merge join functions
            state.join.extend(ext.join);
        }
        // Merge GFM peek handlers
        state.peek_handlers.extend(gfm::gfm_peek_handlers());
    }

    // If tight definitions is enabled, add the join function for it.
    if state.options.tight_definitions {
        state.join.push(join_definition);
    }

    // Serialize the tree.
    let info = Info {
        before: "\n".to_string(),
        after: "\n".to_string(),
        line: 1,
        column: 1,
        line_shift: 0,
    };

    let mut result = state.handle(tree, None, &info);

    // Ensure the result ends with a newline.
    if !result.is_empty() && !result.ends_with('\n') && !result.ends_with('\r') {
        result.push('\n');
    }

    result
}

/// Join function for tight definitions.
///
/// When `tightDefinitions` is enabled, adjacent definitions are joined
/// without a blank line between them.
fn join_definition(left: &Node, right: &Node, _parent: &Node, _state: &State) -> Option<i32> {
    if matches!(left, Node::Definition(_)) && matches!(right, Node::Definition(_)) {
        Some(0)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::mdast::*;

    #[test]
    fn test_empty_tree() {
        let tree = Node::Root(Root {
            children: vec![],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "");
    }

    #[test]
    fn test_to_markdown_returns_string() {
        let tree = Node::Root(Root {
            children: vec![],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert!(result.is_empty() || result.ends_with('\n'));
    }

    #[test]
    fn test_default_options() {
        let opts = Options::default();
        assert_eq!(opts.bullet, '*');
        assert_eq!(opts.emphasis, '*');
        assert_eq!(opts.strong, '*');
        assert_eq!(opts.fence, '`');
        assert!(opts.fences);
        assert!(opts.increment_list_marker);
        assert_eq!(opts.list_item_indent, "one");
        assert_eq!(opts.quote, '"');
        assert_eq!(opts.rule, '*');
        assert_eq!(opts.rule_repetition, 3);
        assert!(!opts.setext);
        assert!(!opts.close_atx);
        assert!(!opts.rule_spaces);
        assert!(!opts.tight_definitions);
        assert!(!opts.resource_link);
        assert!(opts.gfm);
    }

    #[test]
    fn smoke_paragraph() {
        let tree = Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::Text(Text {
                    value: "Hello world".into(),
                    position: None,
                })],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "Hello world\n");
    }

    #[test]
    fn smoke_heading() {
        let tree = Node::Root(Root {
            children: vec![Node::Heading(Heading {
                depth: 2,
                children: vec![Node::Text(Text {
                    value: "Title".into(),
                    position: None,
                })],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "## Title\n");
    }

    #[test]
    fn smoke_unordered_list() {
        let tree = Node::Root(Root {
            children: vec![Node::List(List {
                ordered: false,
                start: None,
                spread: false,
                children: vec![
                    Node::ListItem(ListItem {
                        checked: None,
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "one".into(),
                                position: None,
                            })],
                            position: None,
                        })],
                        position: None,
                    }),
                    Node::ListItem(ListItem {
                        checked: None,
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "two".into(),
                                position: None,
                            })],
                            position: None,
                        })],
                        position: None,
                    }),
                ],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "* one\n* two\n");
    }

    #[test]
    fn smoke_emphasis_strong() {
        let tree = Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Emphasis(Emphasis {
                        children: vec![Node::Text(Text {
                            value: "em".into(),
                            position: None,
                        })],
                        position: None,
                    }),
                    Node::Text(Text {
                        value: " and ".into(),
                        position: None,
                    }),
                    Node::Strong(Strong {
                        children: vec![Node::Text(Text {
                            value: "strong".into(),
                            position: None,
                        })],
                        position: None,
                    }),
                ],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "*em* and **strong**\n");
    }

    #[test]
    fn smoke_code_block() {
        let tree = Node::Root(Root {
            children: vec![Node::Code(Code {
                lang: Some("rust".into()),
                meta: None,
                value: "fn main() {}".into(),
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "```rust\nfn main() {}\n```\n");
    }

    #[test]
    fn smoke_inline_code() {
        let tree = Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::InlineCode(InlineCode {
                    value: "code".into(),
                    position: None,
                })],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "`code`\n");
    }

    #[test]
    fn smoke_blockquote() {
        let tree = Node::Root(Root {
            children: vec![Node::Blockquote(Blockquote {
                children: vec![Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: "quoted".into(),
                        position: None,
                    })],
                    position: None,
                })],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "> quoted\n");
    }

    #[test]
    fn smoke_thematic_break() {
        let tree = Node::Root(Root {
            children: vec![Node::ThematicBreak(ThematicBreak { position: None })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "***\n");
    }

    #[test]
    fn smoke_link() {
        let tree = Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::Link(Link {
                    url: "https://example.com".into(),
                    title: None,
                    children: vec![Node::Text(Text {
                        value: "example".into(),
                        position: None,
                    })],
                    position: None,
                })],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "[example](https://example.com)\n");
    }

    #[test]
    fn smoke_image() {
        let tree = Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::Image(Image {
                    url: "img.png".into(),
                    alt: "alt text".into(),
                    title: None,
                    position: None,
                })],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "![alt text](img.png)\n");
    }

    #[test]
    fn smoke_html() {
        let tree = Node::Root(Root {
            children: vec![Node::Html(Html {
                value: "<div>hello</div>".into(),
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "<div>hello</div>\n");
    }

    #[test]
    fn smoke_ordered_list() {
        let tree = Node::Root(Root {
            children: vec![Node::List(List {
                ordered: true,
                start: Some(1),
                spread: false,
                children: vec![
                    Node::ListItem(ListItem {
                        checked: None,
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "first".into(),
                                position: None,
                            })],
                            position: None,
                        })],
                        position: None,
                    }),
                    Node::ListItem(ListItem {
                        checked: None,
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "second".into(),
                                position: None,
                            })],
                            position: None,
                        })],
                        position: None,
                    }),
                ],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "1. first\n2. second\n");
    }

    #[test]
    fn smoke_task_list() {
        let tree = Node::Root(Root {
            children: vec![Node::List(List {
                ordered: false,
                start: None,
                spread: false,
                children: vec![
                    Node::ListItem(ListItem {
                        checked: Some(true),
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "done".into(),
                                position: None,
                            })],
                            position: None,
                        })],
                        position: None,
                    }),
                    Node::ListItem(ListItem {
                        checked: Some(false),
                        spread: false,
                        children: vec![Node::Paragraph(Paragraph {
                            children: vec![Node::Text(Text {
                                value: "todo".into(),
                                position: None,
                            })],
                            position: None,
                        })],
                        position: None,
                    }),
                ],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "* [x] done\n* [ ] todo\n");
    }

    #[test]
    fn smoke_table() {
        let tree = Node::Root(Root {
            children: vec![Node::Table(Table {
                align: vec![AlignKind::Left, AlignKind::Right],
                children: vec![
                    Node::TableRow(TableRow {
                        children: vec![
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "Name".into(),
                                    position: None,
                                })],
                                position: None,
                            }),
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "Value".into(),
                                    position: None,
                                })],
                                position: None,
                            }),
                        ],
                        position: None,
                    }),
                    Node::TableRow(TableRow {
                        children: vec![
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "a".into(),
                                    position: None,
                                })],
                                position: None,
                            }),
                            Node::TableCell(TableCell {
                                children: vec![Node::Text(Text {
                                    value: "1".into(),
                                    position: None,
                                })],
                                position: None,
                            }),
                        ],
                        position: None,
                    }),
                ],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert!(result.contains("| Name"));
        assert!(result.contains("| a"));
        assert!(result.contains("---"));
    }

    #[test]
    fn smoke_strikethrough() {
        let tree = Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::Delete(Delete {
                    children: vec![Node::Text(Text {
                        value: "deleted".into(),
                        position: None,
                    })],
                    position: None,
                })],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "~~deleted~~\n");
    }

    #[test]
    fn smoke_hard_break() {
        let tree = Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "line1".into(),
                        position: None,
                    }),
                    Node::Break(Break { position: None }),
                    Node::Text(Text {
                        value: "line2".into(),
                        position: None,
                    }),
                ],
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "line1\\\nline2\n");
    }

    #[test]
    fn smoke_definition() {
        let tree = Node::Root(Root {
            children: vec![Node::Definition(Definition {
                url: "https://example.com".into(),
                title: Some("Example".into()),
                identifier: "ex".into(),
                label: Some("ex".into()),
                position: None,
            })],
            position: None,
        });
        let result = to_markdown(&tree, &Options::default());
        assert_eq!(result, "[ex]: https://example.com \"Example\"\n");
    }
}

