//! Compatibility tests ported from BlockNote's `utils.test.ts`.
//!
//! Each test builds `Vec<Block>` matching the BlockNote test data,
//! then performs a `blocks → Y.Doc → blocks` round-trip asserting equality.

use yrs_blocknote::{
    Block, BlockContent, BlockType, InlineContent, Props, Styles, TableCell, TableCellProps,
    TableCellType, TableContent, TableRow, blocks_to_doc, blocks_to_markdown, doc_to_blocks,
    markdown_to_blocks_with,
};

// ── Helpers ──────────────────────────────────────────────────

const FRAG: &str = "document-store";

fn roundtrip(blocks: Vec<Block>) {
    let doc = blocks_to_doc(&blocks, FRAG);
    let output = doc_to_blocks(&doc, FRAG).unwrap();
    assert_eq!(output, blocks);
}

/// Default props for text blocks (paragraph, bullet, numbered, checklist, quote, toggle).
fn text_props() -> Props {
    Props {
        background_color: Some("default".into()),
        text_color: Some("default".into()),
        text_alignment: Some("left".into()),
        ..Props::default()
    }
}

fn plain(text: &str) -> InlineContent {
    InlineContent::plain(text)
}

fn cell(text: &str) -> TableCell {
    TableCell {
        cell_type: TableCellType::TableCell,
        props: TableCellProps::default(),
        content: vec![plain(text)],
    }
}

// ── 5.1 + 5.2: BlockNote Y.Doc round-trip tests ─────────────

#[test]
fn roundtrip_simple_paragraphs() {
    let blocks = vec![
        Block {
            id: "1".into(),
            block_type: BlockType::Paragraph,
            props: text_props(),
            content: BlockContent::Inline(vec![plain("First paragraph")]),
            children: vec![],
        },
        Block {
            id: "2".into(),
            block_type: BlockType::Paragraph,
            props: Props {
                text_alignment: Some("center".into()),
                ..text_props()
            },
            content: BlockContent::Inline(vec![plain("Second paragraph")]),
            children: vec![],
        },
    ];
    roundtrip(blocks);
}

#[test]
fn roundtrip_deeply_nested_lists() {
    let blocks = vec![Block {
        id: "1".into(),
        block_type: BlockType::BulletListItem,
        props: text_props(),
        content: BlockContent::Inline(vec![plain("Level 1")]),
        children: vec![Block {
            id: "2".into(),
            block_type: BlockType::BulletListItem,
            props: text_props(),
            content: BlockContent::Inline(vec![plain("Level 2")]),
            children: vec![Block {
                id: "3".into(),
                block_type: BlockType::BulletListItem,
                props: text_props(),
                content: BlockContent::Inline(vec![plain("Level 3")]),
                children: vec![Block {
                    id: "4".into(),
                    block_type: BlockType::BulletListItem,
                    props: text_props(),
                    content: BlockContent::Inline(vec![plain("Level 4")]),
                    children: vec![],
                }],
            }],
        }],
    }];
    roundtrip(blocks);
}

#[test]
fn roundtrip_numbered_lists() {
    let blocks = vec![
        Block {
            id: "1".into(),
            block_type: BlockType::NumberedListItem,
            props: text_props(),
            content: BlockContent::Inline(vec![plain("First item")]),
            children: vec![],
        },
        Block {
            id: "2".into(),
            block_type: BlockType::NumberedListItem,
            props: text_props(),
            content: BlockContent::Inline(vec![plain("Second item")]),
            children: vec![Block {
                id: "3".into(),
                block_type: BlockType::NumberedListItem,
                props: text_props(),
                content: BlockContent::Inline(vec![plain("Nested item")]),
                children: vec![],
            }],
        },
    ];
    roundtrip(blocks);
}

#[test]
fn roundtrip_checklists() {
    let blocks = vec![
        Block {
            id: "1".into(),
            block_type: BlockType::CheckListItem,
            props: Props {
                checked: Some(true),
                ..text_props()
            },
            content: BlockContent::Inline(vec![plain("Completed task")]),
            children: vec![],
        },
        Block {
            id: "2".into(),
            block_type: BlockType::CheckListItem,
            props: Props {
                checked: Some(false),
                ..text_props()
            },
            content: BlockContent::Inline(vec![plain("Pending task")]),
            children: vec![Block {
                id: "3".into(),
                block_type: BlockType::CheckListItem,
                props: Props {
                    checked: Some(false),
                    ..text_props()
                },
                content: BlockContent::Inline(vec![plain("Subtask")]),
                children: vec![],
            }],
        },
    ];
    roundtrip(blocks);
}

#[test]
fn roundtrip_code_blocks() {
    let blocks = vec![
        Block {
            id: "1".into(),
            block_type: BlockType::CodeBlock,
            props: Props {
                language: Some("javascript".into()),
                ..Props::default()
            },
            content: BlockContent::Inline(vec![plain("console.log(\"Hello, world!\");")]),
            children: vec![],
        },
        Block {
            id: "2".into(),
            block_type: BlockType::CodeBlock,
            props: Props {
                language: Some("typescript".into()),
                ..Props::default()
            },
            content: BlockContent::Inline(vec![plain("const x: number = 42;")]),
            children: vec![],
        },
    ];
    roundtrip(blocks);
}

#[test]
fn roundtrip_headings_with_levels() {
    let blocks = vec![
        Block {
            id: "1".into(),
            block_type: BlockType::Heading,
            props: Props {
                level: Some(1),
                is_toggleable: Some(false),
                ..text_props()
            },
            content: BlockContent::Inline(vec![plain("Heading 1")]),
            children: vec![],
        },
        Block {
            id: "2".into(),
            block_type: BlockType::Heading,
            props: Props {
                level: Some(2),
                is_toggleable: Some(false),
                ..text_props()
            },
            content: BlockContent::Inline(vec![plain("Heading 2")]),
            children: vec![],
        },
        Block {
            id: "3".into(),
            block_type: BlockType::Heading,
            props: Props {
                level: Some(3),
                is_toggleable: Some(true),
                ..text_props()
            },
            content: BlockContent::Inline(vec![plain("Toggle Heading 3")]),
            children: vec![Block {
                id: "4".into(),
                block_type: BlockType::Paragraph,
                props: text_props(),
                content: BlockContent::Inline(vec![plain("Content under toggle heading")]),
                children: vec![],
            }],
        },
    ];
    roundtrip(blocks);
}

#[test]
fn roundtrip_inline_styles_and_links() {
    let blocks = vec![
        Block {
            id: "1".into(),
            block_type: BlockType::Paragraph,
            props: text_props(),
            content: BlockContent::Inline(vec![
                InlineContent::styled("Bold ", Styles::default().with_bold()),
                InlineContent::styled("italic ", Styles::default().with_italic()),
                InlineContent::styled(
                    "underline ",
                    Styles {
                        underline: true,
                        ..Styles::default()
                    },
                ),
                InlineContent::styled("strikethrough ", Styles::default().with_strikethrough()),
                InlineContent::styled("code", Styles::default().with_code()),
            ]),
            children: vec![],
        },
        Block {
            id: "2".into(),
            block_type: BlockType::Paragraph,
            props: text_props(),
            content: BlockContent::Inline(vec![InlineContent::link(
                "https://example.com",
                vec![plain("Link text")],
            )]),
            children: vec![],
        },
    ];
    roundtrip(blocks);
}

#[test]
fn roundtrip_table() {
    let blocks = vec![Block {
        id: "1".into(),
        block_type: BlockType::Table,
        props: Props {
            text_color: Some("default".into()),
            ..Props::default()
        },
        content: BlockContent::Table(TableContent {
            column_widths: vec![Some(100), Some(100), Some(100)],
            header_rows: None,
            header_cols: None,
            rows: vec![
                TableRow {
                    cells: vec![
                        TableCell {
                            cell_type: TableCellType::TableCell,
                            props: TableCellProps {
                                colwidth: Some(vec![100]),
                                ..TableCellProps::default()
                            },
                            content: vec![InlineContent::styled(
                                "Header 1",
                                Styles::default().with_bold(),
                            )],
                        },
                        TableCell {
                            cell_type: TableCellType::TableCell,
                            props: TableCellProps {
                                colwidth: Some(vec![100]),
                                ..TableCellProps::default()
                            },
                            content: vec![InlineContent::styled(
                                "Header 2",
                                Styles::default().with_bold(),
                            )],
                        },
                        TableCell {
                            cell_type: TableCellType::TableCell,
                            props: TableCellProps {
                                colwidth: Some(vec![100]),
                                ..TableCellProps::default()
                            },
                            content: vec![InlineContent::styled(
                                "Header 3",
                                Styles::default().with_bold(),
                            )],
                        },
                    ],
                },
                TableRow {
                    cells: vec![cell("Cell 1"), cell("Cell 2"), cell("Cell 3")],
                },
            ],
        }),
        children: vec![],
    }];
    roundtrip(blocks);
}

#[test]
fn roundtrip_image() {
    let blocks = vec![Block {
        id: "img1".into(),
        block_type: BlockType::Image,
        props: Props {
            background_color: Some("default".into()),
            text_alignment: Some("left".into()),
            name: Some("Example".into()),
            url: Some("exampleURL".into()),
            caption: Some("Caption".into()),
            show_preview: Some(true),
            preview_width: Some(256),
            ..Props::default()
        },
        content: BlockContent::None,
        children: vec![],
    }];
    roundtrip(blocks);
}

#[test]
fn roundtrip_divider() {
    let blocks = vec![
        Block {
            id: "1".into(),
            block_type: BlockType::Paragraph,
            props: text_props(),
            content: BlockContent::Inline(vec![plain("Before divider")]),
            children: vec![],
        },
        Block {
            id: "2".into(),
            block_type: BlockType::Divider,
            props: Props::default(),
            content: BlockContent::None,
            children: vec![],
        },
        Block {
            id: "3".into(),
            block_type: BlockType::Paragraph,
            props: text_props(),
            content: BlockContent::Inline(vec![plain("After divider")]),
            children: vec![],
        },
    ];
    roundtrip(blocks);
}

#[test]
fn roundtrip_quote_with_children() {
    let blocks = vec![Block {
        id: "1".into(),
        block_type: BlockType::Quote,
        props: text_props(),
        content: BlockContent::Inline(vec![InlineContent::styled(
            "This is a quote",
            Styles::default().with_italic(),
        )]),
        children: vec![Block {
            id: "2".into(),
            block_type: BlockType::Paragraph,
            props: text_props(),
            content: BlockContent::Inline(vec![plain("Nested in quote")]),
            children: vec![],
        }],
    }];
    roundtrip(blocks);
}

#[test]
fn roundtrip_toggle_list_item_with_children() {
    let blocks = vec![Block {
        id: "1".into(),
        block_type: BlockType::ToggleListItem,
        props: text_props(),
        content: BlockContent::Inline(vec![plain("Toggle item")]),
        children: vec![Block {
            id: "2".into(),
            block_type: BlockType::Paragraph,
            props: text_props(),
            content: BlockContent::Inline(vec![plain("Hidden content")]),
            children: vec![],
        }],
    }];
    roundtrip(blocks);
}

// ── 5.3: Markdown round-trip tests ──────────────────────────

fn test_id_gen() -> impl FnMut() -> String {
    let mut counter = 0u32;
    move || {
        counter += 1;
        format!("test-{counter}")
    }
}

#[test]
fn markdown_roundtrip_gfm_table() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
    let blocks = markdown_to_blocks_with(md, test_id_gen());
    let output = blocks_to_markdown(&blocks).unwrap();
    assert!(output.contains("| A"), "table header preserved");
    assert!(output.contains("| B"), "table header B preserved");
    assert!(output.contains("| 1"), "table body preserved");
    assert!(output.contains("| 2"), "table body 2 preserved");
}

#[test]
fn markdown_roundtrip_blockquote() {
    let md = "> This is a quote\n";
    let blocks = markdown_to_blocks_with(md, test_id_gen());
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].block_type, BlockType::Quote);
    let output = blocks_to_markdown(&blocks).unwrap();
    assert!(output.contains("> This is a quote"), "blockquote preserved");
}

#[test]
fn markdown_roundtrip_basic_blocks() {
    let md = "## Heading\n\nA paragraph.\n\n- bullet item\n";
    let blocks = markdown_to_blocks_with(md, test_id_gen());
    let output = blocks_to_markdown(&blocks).unwrap();
    assert!(output.contains("## Heading"), "heading preserved");
    assert!(output.contains("A paragraph"), "paragraph preserved");
    assert!(output.contains("bullet item"), "list preserved");
}

// ── 5.5: Block JSON serialization tests ─────────────────────

#[test]
fn json_paragraph_with_default_props() {
    let block = Block::new(BlockType::Paragraph, "p1".into()).with_content(vec![plain("hello")]);
    let json = serde_json::to_value(&block).unwrap();

    assert_eq!(json["id"], "p1");
    assert_eq!(json["type"], "paragraph");
    assert_eq!(json["props"]["textColor"], "default");
    assert_eq!(json["props"]["backgroundColor"], "default");
    assert_eq!(json["props"]["textAlignment"], "left");
    // content is an array
    assert!(json["content"].is_array());
    assert_eq!(json["content"][0]["type"], "text");
    assert_eq!(json["content"][0]["text"], "hello");
    // children is an array
    assert!(json["children"].is_array());
    assert_eq!(json["children"].as_array().unwrap().len(), 0);
}

#[test]
fn json_table_block() {
    let block = Block {
        id: "t1".into(),
        block_type: BlockType::Table,
        props: Props::default(),
        content: BlockContent::Table(TableContent {
            column_widths: vec![Some(100), None],
            header_rows: Some(1),
            header_cols: None,
            rows: vec![TableRow {
                cells: vec![cell("A"), cell("B")],
            }],
        }),
        children: vec![],
    };
    let json = serde_json::to_value(&block).unwrap();

    assert_eq!(json["type"], "table");
    let content = &json["content"];
    assert_eq!(content["type"], "tableContent");
    assert_eq!(content["columnWidths"], serde_json::json!([100, null]));
    assert_eq!(content["headerRows"], 1);
    // headerCols should be absent (skip_serializing_if None)
    assert!(content.get("headerCols").is_none());
    assert_eq!(content["rows"][0]["cells"][0]["type"], "tableCell");
    assert_eq!(content["rows"][0]["cells"][0]["content"][0]["text"], "A");
}

#[test]
fn json_link_inline_content() {
    let link = InlineContent::link(
        "https://example.com",
        vec![InlineContent::styled("click", Styles::default().with_bold())],
    );
    let json = serde_json::to_value(&link).unwrap();

    assert_eq!(json["type"], "link");
    assert_eq!(json["href"], "https://example.com");
    assert_eq!(json["content"][0]["type"], "text");
    assert_eq!(json["content"][0]["text"], "click");
    assert_eq!(json["content"][0]["styles"]["bold"], true);
}

#[test]
fn json_image_block_content_is_null() {
    let block = Block::new(BlockType::Image, "img1".into())
        .with_url("photo.png")
        .with_caption("A photo");
    let json = serde_json::to_value(&block).unwrap();

    assert_eq!(json["type"], "image");
    assert!(json["content"].is_null(), "image content should be null");
    assert_eq!(json["props"]["url"], "photo.png");
    assert_eq!(json["props"]["caption"], "A photo");
}

// ── 5.6: Table cell props encode/decode test ────────────────

#[test]
fn table_cell_props_roundtrip() {
    let blocks = vec![Block {
        id: "t1".into(),
        block_type: BlockType::Table,
        props: Props::default(),
        content: BlockContent::Table(TableContent {
            column_widths: vec![Some(200)],
            header_rows: None,
            header_cols: None,
            rows: vec![TableRow {
                cells: vec![TableCell {
                    cell_type: TableCellType::TableCell,
                    props: TableCellProps {
                        colspan: 2,
                        rowspan: 1,
                        colwidth: Some(vec![200]),
                        background_color: "red".into(),
                        text_color: "default".into(),
                        text_alignment: "left".into(),
                    },
                    content: vec![plain("merged cell")],
                }],
            }],
        }),
        children: vec![],
    }];

    let doc = blocks_to_doc(&blocks, FRAG);
    let output = doc_to_blocks(&doc, FRAG).unwrap();

    let table = output[0].content.as_table().expect("expected table");
    let cell = &table.rows[0].cells[0];
    assert_eq!(cell.props.colspan, 2);
    assert_eq!(cell.props.rowspan, 1);
    assert_eq!(cell.props.colwidth, Some(vec![200]));
    assert_eq!(cell.props.background_color, "red");
    assert_eq!(cell.props.text_color, "default");
    assert_eq!(cell.props.text_alignment, "left");
}

// ── Original BlockNote test case (complex) ──────────────────

#[test]
fn roundtrip_original_blocknote_test_case() {
    let blocks = vec![
        Block {
            id: "1".into(),
            block_type: BlockType::Heading,
            props: Props {
                background_color: Some("blue".into()),
                text_color: Some("yellow".into()),
                text_alignment: Some("right".into()),
                level: Some(2),
                is_toggleable: Some(false),
                ..Props::default()
            },
            content: BlockContent::Inline(vec![
                InlineContent::styled(
                    "Heading ",
                    Styles {
                        bold: true,
                        underline: true,
                        ..Styles::default()
                    },
                ),
                InlineContent::styled(
                    "2",
                    Styles::default().with_italic().with_strikethrough(),
                ),
            ]),
            children: vec![
                Block {
                    id: "2".into(),
                    block_type: BlockType::Paragraph,
                    props: Props {
                        background_color: Some("red".into()),
                        text_alignment: Some("left".into()),
                        text_color: Some("default".into()),
                        ..Props::default()
                    },
                    content: BlockContent::Inline(vec![plain("Paragraph")]),
                    children: vec![],
                },
                Block {
                    id: "3".into(),
                    block_type: BlockType::BulletListItem,
                    props: text_props(),
                    content: BlockContent::Inline(vec![plain("list item")]),
                    children: vec![],
                },
            ],
        },
        Block {
            id: "4".into(),
            block_type: BlockType::Image,
            props: Props {
                background_color: Some("default".into()),
                text_alignment: Some("left".into()),
                name: Some("Example".into()),
                url: Some("exampleURL".into()),
                caption: Some("Caption".into()),
                show_preview: Some(true),
                preview_width: Some(256),
                ..Props::default()
            },
            content: BlockContent::None,
            children: vec![],
        },
        Block {
            id: "5".into(),
            block_type: BlockType::Image,
            props: Props {
                background_color: Some("default".into()),
                text_alignment: Some("left".into()),
                name: Some("Example".into()),
                url: Some("exampleURL".into()),
                caption: Some("Caption".into()),
                show_preview: Some(false),
                preview_width: Some(256),
                ..Props::default()
            },
            content: BlockContent::None,
            children: vec![],
        },
    ];
    roundtrip(blocks);
}

// ── Empty document ──────────────────────────────────────────

#[test]
fn roundtrip_empty_document() {
    let blocks: Vec<Block> = vec![];
    let doc = blocks_to_doc(&blocks, FRAG);
    let output = doc_to_blocks(&doc, FRAG).unwrap();
    assert!(output.is_empty());
}

#[test]
fn debug_ordered_list_roundtrip() {
    let md = "1. **A**: first\n2. **B**: second\n";
    let blocks = yrs_blocknote::markdown_to_blocks(md);
    for (i, b) in blocks.iter().enumerate() {
        eprintln!("block[{i}]: {:?} content={:?}", b.block_type, b.content);
    }
    let output = yrs_blocknote::blocks_to_markdown(&blocks).unwrap();
    eprintln!("---output---\n{output}---end---");
    assert!(output.contains("second"), "second item should be in output");
}

#[test]
fn debug_all_rendering_issues() {
    // Issue 1: consecutive numbered list
    let md1 = "1. First\n2. Second\n3. Third\n";
    let b1 = yrs_blocknote::markdown_to_blocks(md1);
    let o1 = yrs_blocknote::blocks_to_markdown(&b1).unwrap();
    eprintln!("=== Issue 1: Consecutive numbered list ===\nINPUT:\n{md1}OUTPUT:\n{o1}===\n");

    // Issue 2: numbered prefix in heading
    let md2 = "## 1. Problem\n";
    let b2 = yrs_blocknote::markdown_to_blocks(md2);
    let o2 = yrs_blocknote::blocks_to_markdown(&b2).unwrap();
    eprintln!("=== Issue 2: Number in heading ===\nINPUT:\n{md2}OUTPUT:\n{o2}===\n");

    // Issue 3: table separator style
    let md3 = "| A | B |\n|---|---|\n| 1 | 2 |\n";
    let b3 = yrs_blocknote::markdown_to_blocks(md3);
    let o3 = yrs_blocknote::blocks_to_markdown(&b3).unwrap();
    eprintln!("=== Issue 3: Table separator ===\nINPUT:\n{md3}OUTPUT:\n{o3}===\n");

    // Issue 4: tight bullet list with bold
    let md4 = "- **A** — desc\n- **B** — desc\n";
    let b4 = yrs_blocknote::markdown_to_blocks(md4);
    let o4 = yrs_blocknote::blocks_to_markdown(&b4).unwrap();
    eprintln!("=== Issue 4: Tight bullet list ===\nINPUT:\n{md4}OUTPUT:\n{o4}===\n");

    // Issue 5: blockquote
    let md5 = "> Quote line 1\n> Quote line 2\n";
    let b5 = yrs_blocknote::markdown_to_blocks(md5);
    let o5 = yrs_blocknote::blocks_to_markdown(&b5).unwrap();
    eprintln!("=== Issue 5: Blockquote ===\nINPUT:\n{md5}OUTPUT:\n{o5}===\n");
}
