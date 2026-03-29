use yrs_blocknote::{Block, BlockType, InlineContent, Styles, blocks_to_doc, doc_to_blocks};

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
    match &result[0].content[0] {
        InlineContent::Text { text, .. } => assert_eq!(text, "Hello world"),
        InlineContent::HardBreak => panic!("expected text"),
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

    assert_eq!(result[0].content.len(), 2);
    match &result[0].content[0] {
        InlineContent::Text { text, styles } => {
            assert_eq!(text, "Hello ");
            assert!(styles.bold);
        }
        InlineContent::HardBreak => panic!("expected text"),
    }
    match &result[0].content[1] {
        InlineContent::Text { text, styles } => {
            assert_eq!(text, "world");
            assert!(!styles.bold);
        }
        InlineContent::HardBreak => panic!("expected text"),
    }
}

#[test]
fn roundtrip_link() {
    let blocks = vec![
        Block::new(BlockType::Paragraph, "p1".into()).with_content(vec![InlineContent::styled(
            "click here",
            Styles::default().with_link("https://example.com".into()),
        )]),
    ];

    let doc = blocks_to_doc(&blocks, "document-store");
    let result = doc_to_blocks(&doc, "document-store").unwrap();

    match &result[0].content[0] {
        InlineContent::Text { styles, .. } => {
            assert_eq!(styles.link, Some("https://example.com".to_string()));
        }
        InlineContent::HardBreak => panic!("expected text"),
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
