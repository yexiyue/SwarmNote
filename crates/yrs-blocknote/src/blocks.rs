use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use crate::props::Props;
use crate::schema::BlockType;

/// A single `BlockNote` document node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub id: String,
    #[serde(rename = "type")]
    pub block_type: BlockType,
    pub props: Props,
    pub content: BlockContent,
    pub children: Vec<Block>,
}

impl Block {
    /// Create a new block with default props for the given type, empty content and children.
    #[must_use]
    pub fn new(block_type: BlockType, id: String) -> Self {
        let content = if block_type.has_inline_content() {
            BlockContent::Inline(Vec::new())
        } else if block_type == BlockType::Table {
            BlockContent::Table(TableContent::default())
        } else {
            BlockContent::None
        };
        Self {
            id,
            props: block_type.default_props(),
            block_type,
            content,
            children: Vec::new(),
        }
    }

    /// Set the inline content.
    #[must_use]
    pub fn with_inline_content(mut self, content: Vec<InlineContent>) -> Self {
        self.content = BlockContent::Inline(content);
        self
    }

    /// Set the inline content (backward compat alias).
    #[must_use]
    pub fn with_content(mut self, content: Vec<InlineContent>) -> Self {
        self.content = BlockContent::Inline(content);
        self
    }

    /// Set the table content.
    #[must_use]
    pub fn with_table_content(mut self, table: TableContent) -> Self {
        self.content = BlockContent::Table(table);
        self
    }

    /// Set the children blocks.
    #[must_use]
    pub fn with_children(mut self, children: Vec<Block>) -> Self {
        self.children = children;
        self
    }

    // ── Typed prop setters ────────────────────────────────────

    /// Set heading level.
    #[must_use]
    pub fn with_level(mut self, level: u8) -> Self {
        self.props.level = Some(level);
        self
    }

    /// Set checked state (for check list items).
    #[must_use]
    pub fn with_checked(mut self, checked: bool) -> Self {
        self.props.checked = Some(checked);
        self
    }

    /// Set code block language.
    #[must_use]
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.props.language = Some(lang.into());
        self
    }

    /// Set list start number.
    #[must_use]
    pub fn with_start(mut self, start: usize) -> Self {
        self.props.start = Some(start);
        self
    }

    /// Set media URL.
    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.props.url = Some(url.into());
        self
    }

    /// Set media caption.
    #[must_use]
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.props.caption = Some(caption.into());
        self
    }
}

/// Block content — polymorphic: inline text, table, or none.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum BlockContent {
    /// Inline rich text content (paragraph, heading, list items, code block).
    Inline(Vec<InlineContent>),
    /// Table content structure.
    Table(TableContent),
    /// No content (image, divider, etc.).
    #[default]
    None,
}

impl BlockContent {
    /// Returns the inline content slice if this is `Inline`, or empty slice otherwise.
    pub fn as_inline(&self) -> &[InlineContent] {
        match self {
            Self::Inline(v) => v,
            _ => &[],
        }
    }

    /// Returns a mutable reference to the inline content vec if this is `Inline`.
    pub fn as_inline_mut(&mut self) -> Option<&mut Vec<InlineContent>> {
        match self {
            Self::Inline(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the table content if this is `Table`.
    pub fn as_table(&self) -> Option<&TableContent> {
        match self {
            Self::Table(t) => Some(t),
            _ => None,
        }
    }

    /// Returns true if this is `Inline` with an empty vec, or `None`.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Inline(v) => v.is_empty(),
            Self::Table(t) => t.rows.is_empty(),
            Self::None => true,
        }
    }
}

// Custom serde: Inline([...]) → JSON array, Table({...}) → JSON object, None → null
impl Serialize for BlockContent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Inline(items) => items.serialize(serializer),
            Self::Table(table) => {
                #[derive(Serialize)]
                struct TableWrapper<'a> {
                    #[serde(rename = "type")]
                    content_type: &'static str,
                    #[serde(rename = "columnWidths")]
                    column_widths: &'a [Option<u32>],
                    #[serde(rename = "headerRows", skip_serializing_if = "Option::is_none")]
                    header_rows: &'a Option<usize>,
                    #[serde(rename = "headerCols", skip_serializing_if = "Option::is_none")]
                    header_cols: &'a Option<usize>,
                    rows: &'a [TableRow],
                }
                let wrapper = TableWrapper {
                    content_type: "tableContent",
                    column_widths: &table.column_widths,
                    header_rows: &table.header_rows,
                    header_cols: &table.header_cols,
                    rows: &table.rows,
                };
                wrapper.serialize(serializer)
            }
            Self::None => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for BlockContent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BlockContentVisitor;

        impl<'de> Visitor<'de> for BlockContentVisitor {
            type Value = BlockContent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an array (inline content), an object (table content), or null")
            }

            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(BlockContent::None)
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(BlockContent::None)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
                let items =
                    Vec::<InlineContent>::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
                Ok(BlockContent::Inline(items))
            }

            fn visit_map<M: de::MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct TableWrapper {
                    // type field is always "tableContent", we ignore it
                    #[serde(default)]
                    column_widths: Vec<Option<u32>>,
                    #[serde(default)]
                    header_rows: Option<usize>,
                    #[serde(default)]
                    header_cols: Option<usize>,
                    #[serde(default)]
                    rows: Vec<TableRow>,
                }
                let wrapper =
                    TableWrapper::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(BlockContent::Table(TableContent {
                    column_widths: wrapper.column_widths,
                    header_rows: wrapper.header_rows,
                    header_cols: wrapper.header_cols,
                    rows: wrapper.rows,
                }))
            }
        }

        deserializer.deserialize_any(BlockContentVisitor)
    }
}

// ── Table types ──────────────────────────────────────────────

/// Table content structure.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableContent {
    pub column_widths: Vec<Option<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_rows: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_cols: Option<usize>,
    pub rows: Vec<TableRow>,
}


/// A single table row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

/// A single table cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCell {
    #[serde(rename = "type")]
    pub cell_type: TableCellType,
    #[serde(default)]
    pub props: TableCellProps,
    pub content: Vec<InlineContent>,
}

/// Type of a table cell element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TableCellType {
    TableCell,
    TableHeader,
}

fn default_bg() -> String {
    "default".into()
}
fn default_tc() -> String {
    "default".into()
}
fn default_ta() -> String {
    "left".into()
}
fn default_one() -> u32 {
    1
}

/// Properties of a table cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCellProps {
    #[serde(default = "default_bg")]
    pub background_color: String,
    #[serde(default = "default_tc")]
    pub text_color: String,
    #[serde(default = "default_ta")]
    pub text_alignment: String,
    #[serde(default = "default_one")]
    pub colspan: u32,
    #[serde(default = "default_one")]
    pub rowspan: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colwidth: Option<Vec<u32>>,
}

impl Default for TableCellProps {
    fn default() -> Self {
        Self {
            background_color: "default".into(),
            text_color: "default".into(),
            text_alignment: "left".into(),
            colspan: 1,
            rowspan: 1,
            colwidth: None,
        }
    }
}

/// Inline content within a block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum InlineContent {
    Text {
        text: String,
        styles: Styles,
    },
    Link {
        href: String,
        content: Vec<InlineContent>,
    },
    HardBreak,
}

impl InlineContent {
    /// Create a plain text segment with no formatting.
    pub fn plain(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            styles: Styles::default(),
        }
    }

    /// Create a text segment with the given styles.
    pub fn styled(text: impl Into<String>, styles: Styles) -> Self {
        Self::Text {
            text: text.into(),
            styles,
        }
    }

    /// Create a link wrapping text content.
    pub fn link(href: impl Into<String>, content: Vec<InlineContent>) -> Self {
        Self::Link {
            href: href.into(),
            content,
        }
    }
}

/// Inline formatting styles.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Styles {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub underline: bool,
    #[serde(
        rename = "strike",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub strikethrough: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub code: bool,
}

impl Styles {
    #[must_use]
    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    #[must_use]
    pub fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }

    #[must_use]
    pub fn with_strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    #[must_use]
    pub fn with_code(mut self) -> Self {
        self.code = true;
        self
    }

    /// Check if all style fields are at their default (false) values.
    pub fn is_empty(&self) -> bool {
        !self.bold && !self.italic && !self.underline && !self.strikethrough && !self.code
    }
}
