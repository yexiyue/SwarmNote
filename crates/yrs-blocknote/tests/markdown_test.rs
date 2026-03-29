use yrs_blocknote::{BlockType, InlineContent, blocks_to_markdown, markdown_to_blocks_with};

fn test_id_gen() -> impl FnMut() -> String {
    let mut counter = 0u32;
    move || {
        counter += 1;
        format!("test-{counter}")
    }
}

#[test]
fn heading() {
    let blocks = markdown_to_blocks_with("## Hello World\n", test_id_gen());
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].block_type, BlockType::Heading);
    assert_eq!(blocks[0].props.level, Some(2));
    match &blocks[0].content[0] {
        yrs_blocknote::InlineContent::Text { text, .. } => assert_eq!(text, "Hello World"),
        InlineContent::HardBreak => panic!("expected text"),
    }
}

#[test]
fn paragraph_with_bold_and_italic() {
    let blocks = markdown_to_blocks_with("Hello **bold** and *italic*\n", test_id_gen());
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].block_type, BlockType::Paragraph);
    assert!(blocks[0].content.len() >= 3);
    let has_bold = blocks[0]
        .content
        .iter()
        .any(|c| matches!(c, yrs_blocknote::InlineContent::Text { styles, .. } if styles.bold));
    assert!(has_bold);
    let has_italic = blocks[0]
        .content
        .iter()
        .any(|c| matches!(c, yrs_blocknote::InlineContent::Text { styles, .. } if styles.italic));
    assert!(has_italic);
}

#[test]
fn bullet_list() {
    let blocks = markdown_to_blocks_with("- item one\n- item two\n", test_id_gen());
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].block_type, BlockType::BulletListItem);
    assert_eq!(blocks[1].block_type, BlockType::BulletListItem);
}

#[test]
fn ordered_list() {
    let blocks = markdown_to_blocks_with("1. first\n2. second\n", test_id_gen());
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].block_type, BlockType::NumberedListItem);
}

#[test]
fn task_list() {
    let blocks = markdown_to_blocks_with("- [x] done\n- [ ] todo\n", test_id_gen());
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].block_type, BlockType::CheckListItem);
    assert_eq!(blocks[0].props.checked, Some(true));
    assert_eq!(blocks[1].props.checked, Some(false));
}

#[test]
fn code_block() {
    let blocks = markdown_to_blocks_with("```rust\nfn main() {}\n```\n", test_id_gen());
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].block_type, BlockType::CodeBlock);
    assert_eq!(blocks[0].props.language.as_deref(), Some("rust"));
}

#[test]
fn image() {
    let blocks = markdown_to_blocks_with("![alt text](image.png)\n", test_id_gen());
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].block_type, BlockType::Image);
    assert_eq!(blocks[0].props.url.as_deref(), Some("image.png"));
    assert_eq!(blocks[0].props.caption.as_deref(), Some("alt text"));
}

#[test]
fn divider() {
    let blocks = markdown_to_blocks_with("---\n", test_id_gen());
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].block_type, BlockType::Divider);
}

#[test]
fn table() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
    let blocks = markdown_to_blocks_with(md, test_id_gen());
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].block_type, BlockType::Table);
    assert_eq!(blocks[0].children.len(), 2);
}

#[test]
fn nested_list() {
    let md = "- parent\n  - child\n";
    let blocks = markdown_to_blocks_with(md, test_id_gen());
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].block_type, BlockType::BulletListItem);
    assert_eq!(blocks[0].children.len(), 1);
    assert_eq!(blocks[0].children[0].block_type, BlockType::BulletListItem);
}

#[test]
fn inline_link() {
    let blocks = markdown_to_blocks_with("Click [here](https://example.com)\n", test_id_gen());
    let has_link = blocks[0].content.iter().any(|c| {
        matches!(c, yrs_blocknote::InlineContent::Text { styles, .. } if styles.link == Some("https://example.com".to_string()))
    });
    assert!(has_link);
}

#[test]
fn roundtrip_heading() {
    let blocks = markdown_to_blocks_with("## Hello **World**\n", test_id_gen());
    let output = blocks_to_markdown(&blocks).unwrap();
    assert!(output.contains("##"));
    assert!(output.contains("**World**"));
}

#[test]
fn roundtrip_complex_document() {
    let md = "# Title\n\nSome **bold** and *italic*.\n\n- bullet\n\n1. first\n\n- [x] done\n- [ ] pending\n\n```rust\nfn main() {}\n```\n\n---\n\n![image](pic.png)\n\n| A | B |\n|---|---|\n| 1 | 2 |\n";
    let blocks = markdown_to_blocks_with(md, test_id_gen());
    let output = blocks_to_markdown(&blocks).unwrap();

    assert!(output.contains("# Title"));
    assert!(output.contains("**bold**"));
    assert!(output.contains("- bullet"));
    assert!(output.contains("[x]"));
    assert!(output.contains("```rust"));
    assert!(output.contains("---"));
    assert!(output.contains("![image](pic.png)"));
    assert!(output.contains("| A | B |"));
}
