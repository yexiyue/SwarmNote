use yrs_blocknote::{
    Block, BlockType, InlineContent, Styles, blocks_to_doc, doc_to_blocks, doc_to_markdown,
    markdown_to_doc, replace_doc_content,
};

fn make_paragraph(id: &str, text: &str) -> Block {
    Block::new(BlockType::Paragraph, id.into()).with_content(vec![InlineContent::plain(text)])
}

#[test]
fn roundtrip_paragraph() {
    let blocks = vec![make_paragraph("p1", "Hello world")];
    let doc = blocks_to_doc(&blocks, "document-store");
    let result = doc_to_blocks(&doc, "document-store").unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].block_type, BlockType::Paragraph);
    assert_eq!(result[0].id, "p1");
    match &result[0].content.as_inline()[0] {
        InlineContent::Text { text, .. } => assert_eq!(text, "Hello world"),
        _ => panic!("expected text"),
    }
}

#[test]
fn roundtrip_heading_with_props() {
    let mut block = Block::new(BlockType::Heading, "h1".into())
        .with_level(2)
        .with_content(vec![InlineContent::plain("Title")]);
    block.props.text_color = Some("red".into());
    block.props.text_alignment = Some("center".into());

    let doc = blocks_to_doc(&[block], "document-store");
    let result = doc_to_blocks(&doc, "document-store").unwrap();

    assert_eq!(result[0].props.level, Some(2));
    assert_eq!(result[0].props.text_color.as_deref(), Some("red"));
    assert_eq!(result[0].props.text_alignment.as_deref(), Some("center"));
}

#[test]
fn roundtrip_bold_text() {
    let blocks = vec![
        Block::new(BlockType::Paragraph, "p1".into()).with_content(vec![
            InlineContent::styled("Hello ", Styles::default().with_bold()),
            InlineContent::plain("world"),
        ]),
    ];

    let doc = blocks_to_doc(&blocks, "document-store");
    let result = doc_to_blocks(&doc, "document-store").unwrap();

    let inlines = result[0].content.as_inline();
    assert_eq!(inlines.len(), 2);
    match &inlines[0] {
        InlineContent::Text { text, styles } => {
            assert_eq!(text, "Hello ");
            assert!(styles.bold);
        }
        _ => panic!("expected text"),
    }
    match &inlines[1] {
        InlineContent::Text { text, styles } => {
            assert_eq!(text, "world");
            assert!(!styles.bold);
        }
        _ => panic!("expected text"),
    }
}

#[test]
fn roundtrip_link() {
    let blocks = vec![
        Block::new(BlockType::Paragraph, "p1".into()).with_content(vec![InlineContent::link(
            "https://example.com",
            vec![InlineContent::plain("click here")],
        )]),
    ];

    let doc = blocks_to_doc(&blocks, "document-store");
    let result = doc_to_blocks(&doc, "document-store").unwrap();

    let inlines = result[0].content.as_inline();
    match &inlines[0] {
        InlineContent::Link { href, content } => {
            assert_eq!(href, "https://example.com");
            match &content[0] {
                InlineContent::Text { text, .. } => assert_eq!(text, "click here"),
                _ => panic!("expected text inside link"),
            }
        }
        _ => panic!("expected link"),
    }
}

#[test]
fn roundtrip_nested_children() {
    let blocks = vec![
        Block::new(BlockType::BulletListItem, "list1".into())
            .with_content(vec![InlineContent::plain("parent")])
            .with_children(vec![make_paragraph("child1", "child paragraph")]),
    ];

    let doc = blocks_to_doc(&blocks, "document-store");
    let result = doc_to_blocks(&doc, "document-store").unwrap();

    assert_eq!(result[0].children.len(), 1);
    assert_eq!(result[0].children[0].block_type, BlockType::Paragraph);
    assert_eq!(result[0].children[0].id, "child1");
}

#[test]
fn roundtrip_preserves_id() {
    let blocks = vec![make_paragraph("my-custom-id", "text")];
    let doc = blocks_to_doc(&blocks, "document-store");
    let result = doc_to_blocks(&doc, "document-store").unwrap();
    assert_eq!(result[0].id, "my-custom-id");
}

#[test]
fn roundtrip_divider() {
    let blocks = vec![Block::new(BlockType::Divider, "d1".into())];

    let doc = blocks_to_doc(&blocks, "document-store");
    let result = doc_to_blocks(&doc, "document-store").unwrap();

    assert_eq!(result[0].block_type, BlockType::Divider);
    assert!(result[0].content.is_empty());
}

#[test]
fn roundtrip_image() {
    let blocks = vec![
        Block::new(BlockType::Image, "img1".into())
            .with_url("photo.png")
            .with_caption("A photo"),
    ];

    let doc = blocks_to_doc(&blocks, "document-store");
    let result = doc_to_blocks(&doc, "document-store").unwrap();

    assert_eq!(result[0].block_type, BlockType::Image);
    assert_eq!(result[0].props.url.as_deref(), Some("photo.png"));
    assert_eq!(result[0].props.caption.as_deref(), Some("A photo"));
}

// ── replace_doc_content tests ────────────────────────────────

const FRAG: &str = "document-store";

#[test]
fn replace_content_replaces_all_blocks() {
    let doc = blocks_to_doc(&[make_paragraph("p1", "old content")], FRAG);

    replace_doc_content(&doc, "## New Heading\n", FRAG);

    let result = doc_to_blocks(&doc, FRAG).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].block_type, BlockType::Heading);
    match &result[0].content.as_inline()[0] {
        InlineContent::Text { text, .. } => assert_eq!(text, "New Heading"),
        _ => panic!("expected text"),
    }
}

#[test]
fn replace_content_on_empty_doc() {
    let doc = yrs::Doc::new();
    doc.get_or_insert_xml_fragment(FRAG);

    replace_doc_content(&doc, "Hello\n", FRAG);

    let result = doc_to_blocks(&doc, FRAG).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].block_type, BlockType::Paragraph);
}

#[test]
fn replace_content_markdown_roundtrip() {
    let original_md = "## Title\n\nSome paragraph\n\n- item 1\n- item 2\n";
    let doc = markdown_to_doc(original_md, FRAG);

    let new_md = "New paragraph\n\n> this is a quote\n";
    replace_doc_content(&doc, new_md, FRAG);

    let output = doc_to_markdown(&doc, FRAG).unwrap();
    assert!(output.contains("New paragraph"));
    assert!(!output.contains("Title"));
}

#[test]
fn replace_content_multiple_times() {
    let doc = markdown_to_doc("first\n", FRAG);

    replace_doc_content(&doc, "second\n", FRAG);
    replace_doc_content(&doc, "third\n", FRAG);

    let result = doc_to_blocks(&doc, FRAG).unwrap();
    assert_eq!(result.len(), 1);
    match &result[0].content.as_inline()[0] {
        InlineContent::Text { text, .. } => assert_eq!(text, "third"),
        _ => panic!("expected text"),
    }
}
