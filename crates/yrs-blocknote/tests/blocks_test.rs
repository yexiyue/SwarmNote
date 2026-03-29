use yrs_blocknote::{Block, BlockType, InlineContent, Props, Styles};

#[test]
fn block_serializes_to_blocknote_json() {
    let block = Block::new(BlockType::Heading, "abc123".into())
        .with_level(2)
        .with_content(vec![InlineContent::styled(
            "Hello",
            Styles::default().with_bold(),
        )]);

    let json = serde_json::to_value(&block).unwrap();
    assert_eq!(json["id"], "abc123");
    assert_eq!(json["type"], "heading");
    assert_eq!(json["props"]["level"], 2); // now a number, not "2"
    assert_eq!(json["content"][0]["type"], "text");
    assert_eq!(json["content"][0]["text"], "Hello");
    assert_eq!(json["content"][0]["styles"]["bold"], true);
    assert!(json["content"][0]["styles"].get("italic").is_none());
}

#[test]
fn styles_omits_false_and_none_fields() {
    let styles = Styles::default().with_bold();
    let json = serde_json::to_value(&styles).unwrap();
    assert_eq!(json, serde_json::json!({"bold": true}));
}

#[test]
fn styles_includes_link() {
    let styles = Styles::default().with_link("https://example.com".into());
    let json = serde_json::to_value(&styles).unwrap();
    assert_eq!(json, serde_json::json!({"link": "https://example.com"}));
}

#[test]
fn empty_styles_serialize_to_empty_object() {
    let styles = Styles::default();
    let json = serde_json::to_value(&styles).unwrap();
    assert_eq!(json, serde_json::json!({}));
}

#[test]
fn hard_break_serializes_correctly() {
    let hb = InlineContent::HardBreak;
    let json = serde_json::to_value(&hb).unwrap();
    assert_eq!(json, serde_json::json!({"type": "hardBreak"}));
}

#[test]
fn block_roundtrip_serde() {
    let block = Block::new(BlockType::Paragraph, "xyz".into()).with_content(vec![
        InlineContent::styled("Hello ", Styles::default().with_bold()),
        InlineContent::HardBreak,
        InlineContent::styled(
            "world",
            Styles::default()
                .with_italic()
                .with_link("https://example.com".into()),
        ),
    ]);

    let json_str = serde_json::to_string(&block).unwrap();
    let deserialized: Block = serde_json::from_str(&json_str).unwrap();
    assert_eq!(block, deserialized);
}

#[test]
fn block_type_serde_camel_case() {
    let json = serde_json::to_value(BlockType::BulletListItem).unwrap();
    assert_eq!(json, "bulletListItem");

    let parsed: BlockType = serde_json::from_value(serde_json::json!("checkListItem")).unwrap();
    assert_eq!(parsed, BlockType::CheckListItem);
}

#[test]
fn block_type_strum_roundtrip() {
    use std::str::FromStr;
    let bt = BlockType::NumberedListItem;
    let s = bt.to_string();
    assert_eq!(s, "numberedListItem");
    let parsed = BlockType::from_str(&s).unwrap();
    assert_eq!(parsed, bt);
}

#[test]
fn props_typed_fields() {
    let props = Props {
        level: Some(3),
        checked: Some(true),
        language: Some("rust".into()),
        url: Some("img.png".into()),
        caption: Some("A photo".into()),
        start: Some(5),
        ..Props::default()
    };

    assert_eq!(props.level, Some(3));
    assert_eq!(props.checked, Some(true));
    assert_eq!(props.language.as_deref(), Some("rust"));
    assert_eq!(props.url.as_deref(), Some("img.png"));
    assert_eq!(props.caption.as_deref(), Some("A photo"));
    assert_eq!(props.start, Some(5));
}

#[test]
fn props_json_correct_types() {
    let block = Block::new(BlockType::Heading, "h1".into()).with_level(2);
    let json = serde_json::to_value(&block).unwrap();
    // level should be a number, not a string
    assert!(json["props"]["level"].is_number());
    assert_eq!(json["props"]["level"], 2);
    // isToggleable should be a boolean
    assert!(json["props"]["isToggleable"].is_boolean());
    assert_eq!(json["props"]["isToggleable"], false);
}
