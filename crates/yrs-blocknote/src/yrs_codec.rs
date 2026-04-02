use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use yrs::types::Attrs;
use yrs::types::text::YChange;
use yrs::{
    Any, Doc, OffsetKind, Options, Out, Text, Transact, TransactionMut, Xml, XmlElementPrelim,
    XmlElementRef, XmlFragment, XmlOut, XmlTextPrelim,
};

use crate::blocks::{
    Block, BlockContent, InlineContent, Styles, TableCell, TableCellProps, TableCellType,
    TableContent, TableRow,
};
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
        attrs
    }

    pub(crate) fn from_yrs_attrs(attrs: &Attrs) -> Self {
        let mut styles = Self::default();
        for key in attrs.keys() {
            match key.as_ref() {
                "bold" => styles.bold = true,
                "italic" => styles.italic = true,
                "underline" => styles.underline = true,
                "strike" => styles.strikethrough = true,
                "code" => styles.code = true,
                _ => {}
            }
        }
        styles
    }
}

/// Extract link href from yrs attrs if present.
fn extract_link_href(attrs: &Attrs) -> Option<String> {
    if let Some(Any::Map(map)) = attrs.get(&Arc::from("link")) {
        if let Some(Any::String(href)) = map.get("href") {
            return Some(href.to_string());
        }
    }
    None
}

/// Build link yrs attributes: `{ href: "url" }` map.
fn link_yrs_attr(href: &str) -> Any {
    let map = HashMap::from([("href".to_string(), Any::String(Arc::from(href)))]);
    Any::Map(Arc::new(map))
}

// ── Blocks → Y.Doc ────────────────────────────────────────────

pub(crate) fn blocks_to_doc(
    blocks: &[Block],
    fragment_name: &str,
    mut id_gen: impl FnMut() -> String,
) -> Doc {
    let doc = Doc::with_options(Options {
        offset_kind: OffsetKind::Utf16,
        ..Options::default()
    });
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

    match &block.content {
        BlockContent::Inline(inlines) if block.block_type.has_inline_content() => {
            if !inlines.is_empty() {
                let text_ref = content_elem.push_back(txn, XmlTextPrelim::new(""));
                encode_inline_content(inlines, &text_ref, txn);
            }
        }
        BlockContent::Table(table) => {
            encode_table(&content_elem, txn, table);
        }
        _ => {}
    }

    if !block.children.is_empty() {
        let child_group = container.push_back(txn, XmlElementPrelim::empty(BLOCK_GROUP));
        for child in &block.children {
            encode_block(&child_group, txn, child, id_gen);
        }
    }
}

/// Encode inline content (including links) into an `XmlText` node.
fn encode_inline_content(
    inlines: &[InlineContent],
    text_ref: &yrs::XmlTextRef,
    txn: &mut TransactionMut<'_>,
) {
    let mut offset = 0u32;
    for inline in inlines {
        match inline {
            InlineContent::Text { text, styles } => {
                let attrs = styles.to_yrs_attrs();
                text_ref.insert_with_attributes(txn, offset, text, attrs);
                offset += text.encode_utf16().count() as u32;
            }
            InlineContent::Link { href, content } => {
                // For link, encode each inner text with the link attribute added
                for inner in content {
                    if let InlineContent::Text { text, styles } = inner {
                        let mut attrs = styles.to_yrs_attrs();
                        attrs.insert(Arc::from("link"), link_yrs_attr(href));
                        text_ref.insert_with_attributes(txn, offset, text, attrs);
                        offset += text.encode_utf16().count() as u32;
                    }
                }
            }
            InlineContent::HardBreak => {
                // HardBreak in inline content — skip for now
            }
        }
    }
}

/// Encode table content into a `<table>` element.
fn encode_table(
    table_elem: &XmlElementRef,
    txn: &mut TransactionMut<'_>,
    table: &TableContent,
) {
    for row in &table.rows {
        let row_elem = table_elem.push_back(txn, XmlElementPrelim::empty("tableRow"));
        for cell in &row.cells {
            let cell_tag = match cell.cell_type {
                TableCellType::TableCell => "tableCell",
                TableCellType::TableHeader => "tableHeader",
            };
            let cell_elem = row_elem.push_back(txn, XmlElementPrelim::empty(cell_tag));

            // Write cell props as XML attributes
            cell_elem.insert_attribute(txn, "backgroundColor", cell.props.background_color.as_str());
            cell_elem.insert_attribute(txn, "textColor", cell.props.text_color.as_str());
            cell_elem.insert_attribute(txn, "textAlignment", cell.props.text_alignment.as_str());
            cell_elem.insert_attribute(txn, "colspan", cell.props.colspan.to_string());
            cell_elem.insert_attribute(txn, "rowspan", cell.props.rowspan.to_string());
            if let Some(colwidth) = &cell.props.colwidth {
                let colwidth_str = colwidth
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                cell_elem.insert_attribute(txn, "colwidth", colwidth_str);
            }

            // Each cell contains a <tableParagraph> with XmlText
            let para_elem = cell_elem.push_back(txn, XmlElementPrelim::empty("tableParagraph"));
            if !cell.content.is_empty() {
                let text_ref = para_elem.push_back(txn, XmlTextPrelim::new(""));
                encode_inline_content(&cell.content, &text_ref, txn);
            }
        }
    }
}

/// Replace the entire content of an existing Y.Doc's `XmlFragment`.
///
/// Clears the fragment via `remove_range`, then re-encodes the given blocks.
/// Keeps the same Doc instance so CRDT history stays continuous.
pub(crate) fn replace_fragment_content(
    doc: &Doc,
    blocks: &[Block],
    fragment_name: &str,
    mut id_gen: impl FnMut() -> String,
) {
    let fragment = doc.get_or_insert_xml_fragment(fragment_name);
    let mut txn = doc.transact_mut();

    let len = fragment.len(&txn);
    if len > 0 {
        fragment.remove_range(&mut txn, 0, len);
    }

    let block_group = fragment.push_back(&mut txn, XmlElementPrelim::empty(BLOCK_GROUP));
    for block in blocks {
        encode_block(&block_group, &mut txn, block, &mut id_gen);
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

    let content = if block_type == BlockType::Table {
        BlockContent::Table(decode_table(&content_elem, txn))
    } else if block_type.has_inline_content() {
        BlockContent::Inline(decode_inline_content(&content_elem, txn))
    } else {
        BlockContent::None
    };

    // Children are in a blockGroup that may follow the content element
    // For table, blockGroup is at index 1 (after <table>)
    // For inline content blocks, blockGroup could be at index 1 or 2
    let children = find_block_group(container, txn)
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

/// Find the blockGroup child element in a container.
fn find_block_group(
    container: &XmlElementRef,
    txn: &yrs::Transaction<'_>,
) -> Option<XmlElementRef> {
    for child in container.children(txn) {
        if let Some(elem) = child.into_xml_element() {
            if elem.tag().as_ref() == BLOCK_GROUP {
                return Some(elem);
            }
        }
    }
    None
}

/// Decode table content from a `<table>` element.
fn decode_table(table_elem: &XmlElementRef, txn: &yrs::Transaction<'_>) -> TableContent {
    let mut rows = Vec::new();

    for row_child in table_elem.children(txn) {
        let Some(row_elem) = row_child.into_xml_element() else {
            continue;
        };
        if row_elem.tag().as_ref() != "tableRow" {
            continue;
        }

        let mut cells = Vec::new();
        for cell_child in row_elem.children(txn) {
            let Some(cell_elem) = cell_child.into_xml_element() else {
                continue;
            };
            let tag = cell_elem.tag();
            let cell_type = match tag.as_ref() {
                "tableCell" => TableCellType::TableCell,
                "tableHeader" => TableCellType::TableHeader,
                _ => continue,
            };

            let cell_props = decode_cell_props(&cell_elem, txn);
            let content = decode_cell_inline_content(&cell_elem, txn);

            cells.push(TableCell {
                cell_type,
                props: cell_props,
                content,
            });
        }

        rows.push(TableRow { cells });
    }

    // Determine column widths from first row's cells
    let column_widths = if let Some(first_row) = rows.first() {
        first_row
            .cells
            .iter()
            .map(|cell| {
                cell.props
                    .colwidth
                    .as_ref()
                    .and_then(|cw| cw.first().copied())
            })
            .collect()
    } else {
        Vec::new()
    };

    // Determine header rows/cols
    let header_rows = rows
        .iter()
        .take_while(|row| {
            row.cells
                .first()
                .is_some_and(|c| c.cell_type == TableCellType::TableHeader)
        })
        .count();
    let header_cols = rows.first().map_or(0, |row| {
        row.cells
            .iter()
            .take_while(|c| c.cell_type == TableCellType::TableHeader)
            .count()
    });

    TableContent {
        column_widths,
        header_rows: if header_rows > 0 {
            Some(header_rows)
        } else {
            None
        },
        header_cols: if header_cols > 0 {
            Some(header_cols)
        } else {
            None
        },
        rows,
    }
}

/// Parse cell props from XML attributes.
fn decode_cell_props(cell_elem: &XmlElementRef, txn: &yrs::Transaction<'_>) -> TableCellProps {
    let mut props = TableCellProps::default();
    for (key, value) in cell_elem.attributes(txn) {
        let Some(s) = out_to_string(&value) else {
            continue;
        };
        match key {
            "backgroundColor" => props.background_color = s,
            "textColor" => props.text_color = s,
            "textAlignment" => props.text_alignment = s,
            "colspan" => {
                if let Ok(v) = s.parse() {
                    props.colspan = v;
                }
            }
            "rowspan" => {
                if let Ok(v) = s.parse() {
                    props.rowspan = v;
                }
            }
            "colwidth" => {
                let widths: Vec<u32> = s.split(',').filter_map(|w| w.trim().parse().ok()).collect();
                if !widths.is_empty() {
                    props.colwidth = Some(widths);
                }
            }
            _ => {}
        }
    }
    props
}

/// Decode inline content from a cell element (looks for <tableParagraph> children).
fn decode_cell_inline_content(
    cell_elem: &XmlElementRef,
    txn: &yrs::Transaction<'_>,
) -> Vec<InlineContent> {
    for child in cell_elem.children(txn) {
        if let Some(elem) = child.into_xml_element() {
            if elem.tag().as_ref() == "tableParagraph" {
                return decode_inline_content(&elem, txn);
            }
        }
    }
    Vec::new()
}

fn decode_inline_content(
    content_elem: &XmlElementRef,
    txn: &yrs::Transaction<'_>,
) -> Vec<InlineContent> {
    let mut result = Vec::new();

    for child in content_elem.children(txn) {
        match child {
            XmlOut::Text(text_ref) => {
                decode_text_with_links(&text_ref, txn, &mut result);
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

/// Decode text diffs, grouping consecutive segments with the same link href
/// into `InlineContent::Link` wrappers.
fn decode_text_with_links(
    text_ref: &yrs::XmlTextRef,
    txn: &yrs::Transaction<'_>,
    result: &mut Vec<InlineContent>,
) {
    // Collect all segments, grouping consecutive segments with the same link href.
    let mut pending_link_href: Option<String> = None;
    let mut pending_link_content: Vec<InlineContent> = Vec::new();

    for diff in text_ref.diff(txn, YChange::identity) {
        let Out::Any(Any::String(s)) = &diff.insert else {
            continue;
        };

        let attrs = diff.attributes.as_ref();
        let link_href = attrs.and_then(|a| extract_link_href(a));
        let styles = attrs
            .map(|a| Styles::from_yrs_attrs(a))
            .unwrap_or_default();

        let text_item = InlineContent::Text {
            text: s.to_string(),
            styles,
        };

        let same_link = match (&pending_link_href, &link_href) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        };

        if same_link {
            // Continue accumulating into the same link
            pending_link_content.push(text_item);
        } else {
            // Flush pending link if any
            if let Some(href) = pending_link_href.take() {
                let content = std::mem::take(&mut pending_link_content);
                result.push(InlineContent::Link { href, content });
            }

            if let Some(href) = link_href {
                // Start a new link
                pending_link_href = Some(href);
                pending_link_content.push(text_item);
            } else {
                // Plain text
                result.push(text_item);
            }
        }
    }

    // Flush remaining link
    if let Some(href) = pending_link_href {
        result.push(InlineContent::Link {
            href,
            content: pending_link_content,
        });
    }
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
