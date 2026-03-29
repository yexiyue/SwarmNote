use yrs_blocknote::{
    BlockType, doc_to_blocks, doc_to_markdown, markdown_to_blocks_with, markdown_to_doc,
};

#[test]
fn md_to_doc_to_md_roundtrip() {
    let md = r"# Title

Hello **bold** and *italic* text.

- bullet one
- bullet two

1. first
2. second

- [x] done
- [ ] pending

```rust
fn main() {}
```

---

![photo](pic.png)

| A | B |
|---|---|
| 1 | 2 |
";

    let doc = markdown_to_doc(md, "document-store");
    let output = doc_to_markdown(&doc, "document-store").unwrap();

    assert!(output.contains("# Title"), "heading preserved");
    assert!(output.contains("**bold**"), "bold preserved");
    assert!(output.contains("*italic*"), "italic preserved");
    assert!(output.contains("bullet"), "bullet list preserved");
    assert!(output.contains("1."), "ordered list preserved");
    assert!(output.contains("[x]"), "checked task preserved");
    assert!(output.contains("[ ]"), "unchecked task preserved");
    assert!(output.contains("```rust"), "code fence preserved");
    assert!(output.contains("fn main()"), "code content preserved");
    assert!(output.contains("---"), "divider preserved");
    assert!(output.contains("![photo](pic.png)"), "image preserved");
    // TODO: table yrs roundtrip requires specialized encoding (not blockContainer-wrapped)
    // assert!(output.contains("| A |"), "table preserved");
}

#[test]
fn md_to_doc_to_blocks_preserves_structure() {
    let md = "## Hello\n\nWorld\n";
    let doc = markdown_to_doc(md, "test-frag");
    let blocks = doc_to_blocks(&doc, "test-frag").unwrap();

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].block_type, BlockType::Heading);
    assert_eq!(blocks[0].props.level, Some(2));
    assert_eq!(blocks[1].block_type, BlockType::Paragraph);
}

#[test]
fn blocks_ids_are_unique() {
    let blocks =
        markdown_to_blocks_with("# A\n\n# B\n\n# C\n", yrs_blocknote::default_id_generator);
    let ids: Vec<_> = blocks.iter().map(|b| &b.id).collect();
    for (i, id) in ids.iter().enumerate() {
        for (j, other) in ids.iter().enumerate() {
            if i != j {
                assert_ne!(id, other, "IDs must be unique");
            }
        }
    }
}

#[test]
fn doc_to_blocks_returns_empty_for_empty_fragment() {
    let doc = yrs::Doc::new();
    let result = yrs_blocknote::doc_to_blocks(&doc, "nonexistent").unwrap();
    assert!(result.is_empty(), "empty fragment should return empty blocks");
}

#[test]
fn doc_to_blocks_returns_invalid_schema_for_wrong_root() {
    use yrs::{Transact, XmlElementPrelim, XmlFragment};
    let doc = yrs::Doc::new();
    {
        let fragment = doc.get_or_insert_xml_fragment("bad");
        let mut txn = doc.transact_mut();
        fragment.push_back(&mut txn, XmlElementPrelim::empty("notBlockGroup"));
    }
    let result = yrs_blocknote::doc_to_blocks(&doc, "bad");
    assert!(result.is_err());
    assert!(
        result.unwrap_err().to_string().contains("blockGroup"),
        "error should mention expected root element"
    );
}
