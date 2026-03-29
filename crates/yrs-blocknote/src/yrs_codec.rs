use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use yrs::types::Attrs;
use yrs::types::text::YChange;
use yrs::{
    Any, Doc, Out, Text, Transact, TransactionMut, Xml, XmlElementPrelim, XmlElementRef,
    XmlFragment, XmlOut, XmlTextPrelim,
};

use crate::blocks::{Block, InlineContent, Styles};
use crate::props::Props;
use crate::schema::{BLOCK_CONTAINER, BLOCK_GROUP, BlockType};

static EMPTY_ANY_MAP: LazyLock<Any> = LazyLock::new(|| Any::Map(Arc::new(HashMap::new())));

fn yrs_flag() -> Any {
    EMPTY_ANY_MAP.clone()
}

// ── Props ↔ yrs attributes ───────────────────────────────────

impl Props {
    /// Write all props as yrs XML attributes to the given element.
    pub(crate) fn write_to_yrs(&self, elem: &XmlElementRef, txn: &mut TransactionMut<'_>) {
        macro_rules! write_str {
            ($field:expr, $key:expr) => {
                if let Some(v) = &$field {
                    elem.insert_attribute(txn, $key, v.as_str());
                }
            };
        }
        macro_rules! write_display {
            ($field:expr, $key:expr) => {
                if let Some(v) = &$field {
                    elem.insert_attribute(txn, $key, v.to_string());
                }
            };
        }
        write_str!(self.text_color, "textColor");
        write_str!(self.background_color, "backgroundColor");
        write_str!(self.text_alignment, "textAlignment");
        write_display!(self.level, "level");
        write_display!(self.is_toggleable, "isToggleable");
        write_display!(self.checked, "checked");
        write_display!(self.start, "start");
        write_str!(self.language, "language");
        write_str!(self.name, "name");
        write_str!(self.url, "url");
        write_str!(self.caption, "caption");
        write_display!(self.show_preview, "showPreview");
        write_display!(self.preview_width, "previewWidth");
        for (k, v) in &self.other {
            elem.insert_attribute(txn, k.as_str(), v.as_str());
        }
    }

    /// Parse props from yrs XML element attributes.
    pub(crate) fn from_yrs_element(elem: &XmlElementRef, txn: &yrs::Transaction<'_>) -> Self {
        let mut props = Self::default();
        for (key, value) in elem.attributes(txn) {
            let Some(s) = out_to_string(&value) else {
                continue;
            };
            match key {
                "textColor" => props.text_color = Some(s),
                "backgroundColor" => props.background_color = Some(s),
                "textAlignment" => props.text_alignment = Some(s),
                "level" => props.level = s.parse().ok(),
                "isToggleable" => props.is_toggleable = Some(s == "true"),
                "checked" => props.checked = Some(s == "true"),
                "start" => props.start = s.parse().ok(),
                "language" => props.language = Some(s),
                "name" => props.name = Some(s),
                "url" => props.url = Some(s),
                "caption" => props.caption = Some(s),
                "showPreview" => props.show_preview = Some(s == "true"),
                "previewWidth" => props.preview_width = s.parse().ok(),
                _ => {
                    props.other.insert(key.to_string(), s);
                }
            }
        }
        props
    }
}

// ── Styles ↔ yrs Attrs ───────────────────────────────────────

impl Styles {
    pub(crate) fn to_yrs_attrs(&self) -> Attrs {
        let mut attrs = Attrs::new();
        if self.bold {
            attrs.insert(Arc::from("bold"), yrs_flag());
        }
        if self.italic {
            attrs.insert(Arc::from("italic"), yrs_flag());
        }
        if self.underline {
            attrs.insert(Arc::from("underline"), yrs_flag());
        }
        if self.strikethrough {
            attrs.insert(Arc::from("strike"), yrs_flag());
        }
        if self.code {
            attrs.insert(Arc::from("code"), yrs_flag());
        }
        if let Some(url) = &self.link {
            let map = HashMap::from([("href".to_string(), Any::String(Arc::from(url.as_str())))]);
            attrs.insert(Arc::from("link"), Any::Map(Arc::new(map)));
        }
        attrs
    }

    pub(crate) fn from_yrs_attrs(attrs: &Attrs) -> Self {
        let mut styles = Self::default();
        for (key, value) in attrs {
            match key.as_ref() {
                "bold" => styles.bold = true,
                "italic" => styles.italic = true,
                "underline" => styles.underline = true,
                "strike" => styles.strikethrough = true,
                "code" => styles.code = true,
                "link" => {
                    if let Any::Map(map) = value {
                        if let Some(Any::String(href)) = map.get("href") {
                            styles.link = Some(href.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
        styles
    }
}

// ── Blocks → Y.Doc ────────────────────────────────────────────

pub(crate) fn blocks_to_doc(
    blocks: &[Block],
    fragment_name: &str,
    mut id_gen: impl FnMut() -> String,
) -> Doc {
    let doc = Doc::new();
    let fragment = doc.get_or_insert_xml_fragment(fragment_name);
    let mut txn = doc.transact_mut();

    let block_group = fragment.push_back(&mut txn, XmlElementPrelim::empty(BLOCK_GROUP));
    for block in blocks {
        encode_block(&block_group, &mut txn, block, &mut id_gen);
    }

    drop(txn);
    doc
}

fn encode_block(
    parent_group: &XmlElementRef,
    txn: &mut TransactionMut<'_>,
    block: &Block,
    id_gen: &mut impl FnMut() -> String,
) {
    let container = parent_group.push_back(txn, XmlElementPrelim::empty(BLOCK_CONTAINER));

    let id = if block.id.is_empty() {
        id_gen()
    } else {
        block.id.clone()
    };
    container.insert_attribute(txn, "id", id);

    let props = block.block_type.props_with_defaults(&block.props);
    props.write_to_yrs(&container, txn);

    let tag: &'static str = block.block_type.into();
    let content_elem = container.push_back(txn, XmlElementPrelim::empty(tag));

    // BlockNote expects props on both the container and the content element
    props.write_to_yrs(&content_elem, txn);

    if !block.content.is_empty() && block.block_type.has_inline_content() {
        let text_ref = content_elem.push_back(txn, XmlTextPrelim::new(""));
        let mut offset = 0u32;
        for inline in &block.content {
            if let InlineContent::Text { text, styles } = inline {
                text_ref.insert_with_attributes(txn, offset, text, styles.to_yrs_attrs());
                offset += text.encode_utf16().count() as u32;
            }
        }
    }

    if !block.children.is_empty() {
        let child_group = container.push_back(txn, XmlElementPrelim::empty(BLOCK_GROUP));
        for child in &block.children {
            encode_block(&child_group, txn, child, id_gen);
        }
    }
}

// ── Y.Doc → Blocks ────────────────────────────────────────────

pub(crate) fn doc_to_blocks(doc: &Doc, fragment_name: &str) -> crate::ConvertResult<Vec<Block>> {
    let fragment = doc.get_or_insert_xml_fragment(fragment_name);
    let txn = doc.transact();

    // Empty fragment = empty document, not an error
    if fragment.len(&txn) == 0 {
        return Ok(vec![]);
    }

    let first = fragment.get(&txn, 0).ok_or_else(|| {
        crate::ConvertError::InvalidSchema("fragment reports non-zero len but no children".into())
    })?;
    let block_group = first.into_xml_element().ok_or_else(|| {
        crate::ConvertError::InvalidSchema("root child is not an XmlElement".to_string())
    })?;
    if block_group.tag().as_ref() != BLOCK_GROUP {
        return Err(crate::ConvertError::InvalidSchema(format!(
            "expected '{}', found '{}'",
            BLOCK_GROUP,
            block_group.tag()
        )));
    }

    Ok(decode_block_group(&block_group, &txn))
}

fn decode_block_group(block_group: &XmlElementRef, txn: &yrs::Transaction<'_>) -> Vec<Block> {
    let mut blocks = Vec::new();
    for child in block_group.children(txn) {
        let Some(container) = child.into_xml_element() else {
            continue;
        };
        if container.tag().as_ref() != BLOCK_CONTAINER {
            continue;
        }
        if let Some(block) = decode_block_container(&container, txn) {
            blocks.push(block);
        }
    }
    blocks
}

fn decode_block_container(container: &XmlElementRef, txn: &yrs::Transaction<'_>) -> Option<Block> {
    let id = container
        .get_attribute(txn, "id")
        .and_then(|v| out_to_string(&v))
        .unwrap_or_default();

    let first_child = container.get(txn, 0)?;
    let content_elem = first_child.into_xml_element()?;

    let block_type: BlockType = content_elem.tag().as_ref().parse().ok()?;
    let props = Props::from_yrs_element(&content_elem, txn);
    let content = decode_inline_content(&content_elem, txn);

    let children = container
        .get(txn, 1)
        .and_then(yrs::XmlOut::into_xml_element)
        .filter(|g| g.tag().as_ref() == BLOCK_GROUP)
        .map(|g| decode_block_group(&g, txn))
        .unwrap_or_default();

    Some(Block {
        id,
        block_type,
        props,
        content,
        children,
    })
}

fn decode_inline_content(
    content_elem: &XmlElementRef,
    txn: &yrs::Transaction<'_>,
) -> Vec<InlineContent> {
    let mut result = Vec::new();

    for child in content_elem.children(txn) {
        match child {
            XmlOut::Text(text_ref) => {
                for diff in text_ref.diff(txn, YChange::identity) {
                    if let Out::Any(Any::String(s)) = &diff.insert {
                        let styles = diff
                            .attributes
                            .as_ref()
                            .map(|attrs| Styles::from_yrs_attrs(attrs))
                            .unwrap_or_default();
                        result.push(InlineContent::Text {
                            text: s.to_string(),
                            styles,
                        });
                    }
                }
            }
            XmlOut::Element(elem) => {
                let tag = elem.tag();
                let hard_break: &str = BlockType::HardBreak.into();
                let table_para: &str = BlockType::TableParagraph.into();
                if tag.as_ref() == hard_break {
                    result.push(InlineContent::HardBreak);
                } else if tag.as_ref() == table_para {
                    result.extend(decode_inline_content(&elem, txn));
                }
            }
            XmlOut::Fragment(_) => {}
        }
    }

    result
}

fn out_to_string(out: &Out) -> Option<String> {
    match out {
        Out::Any(Any::String(s)) => Some(s.to_string()),
        Out::Any(Any::Bool(b)) => Some(b.to_string()),
        Out::Any(Any::Number(n)) => Some(n.to_string()),
        Out::Any(Any::BigInt(n)) => Some(n.to_string()),
        _ => None,
    }
}
