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
    pub content: Vec<InlineContent>,
    pub children: Vec<Block>,
}

impl Block {
    /// Create a new block with default props for the given type, empty content and children.
    #[must_use]
    pub fn new(block_type: BlockType, id: String) -> Self {
        Self {
            id,
            props: block_type.default_props(),
            block_type,
            content: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Set the inline content.
    #[must_use]
    pub fn with_content(mut self, content: Vec<InlineContent>) -> Self {
        self.content = content;
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

/// Inline content within a block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum InlineContent {
    Text { text: String, styles: Styles },
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
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub strikethrough: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub code: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
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

    #[must_use]
    pub fn with_link(mut self, url: String) -> Self {
        self.link = Some(url);
        self
    }
}
