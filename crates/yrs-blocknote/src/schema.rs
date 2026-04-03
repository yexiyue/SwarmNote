use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};

use crate::props::Props;

/// `BlockNote` block type identifier.
///
/// Each variant corresponds to a `BlockNote` block type and knows its own
/// default props, inline content support, and child nesting rules.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Display,
    EnumString,
    IntoStaticStr,
    Serialize,
    Deserialize,
)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum BlockType {
    Paragraph,
    Heading,
    BulletListItem,
    NumberedListItem,
    CheckListItem,
    CodeBlock,
    Image,
    Table,
    TableRow,
    TableHeader,
    TableCell,
    TableParagraph,
    Divider,
    HardBreak,
    Quote,
    ToggleListItem,
}

impl BlockType {
    /// Whether this block type supports inline text content (`XmlText` children).
    pub fn has_inline_content(&self) -> bool {
        matches!(
            self,
            Self::Paragraph
                | Self::Heading
                | Self::BulletListItem
                | Self::NumberedListItem
                | Self::CheckListItem
                | Self::CodeBlock
                | Self::Quote
                | Self::ToggleListItem
        )
    }

    /// Whether this block type carries text-level props (textColor, backgroundColor, textAlignment).
    pub fn has_text_props(&self) -> bool {
        matches!(
            self,
            Self::Paragraph
                | Self::Heading
                | Self::BulletListItem
                | Self::NumberedListItem
                | Self::CheckListItem
                | Self::Quote
                | Self::ToggleListItem
        )
    }

    /// Generate the default props for this block type.
    pub fn default_props(&self) -> Props {
        let mut props = Props::default();
        if self.has_text_props() {
            props.insert_text_defaults();
        }
        match self {
            Self::Heading => {
                props.level = Some(1);
                props.is_toggleable = Some(false);
            }
            Self::CheckListItem => {
                props.checked = Some(false);
            }
            _ => {}
        }
        props
    }

    /// Merge user-provided props with this type's defaults (user values take precedence).
    pub fn props_with_defaults(&self, overrides: &Props) -> Props {
        let mut props = self.default_props();
        props.merge_from(overrides);
        props
    }
}

// XML structural element names (not block types — used internally by yrs_codec).
pub(crate) const BLOCK_GROUP: &str = "blockGroup";
pub(crate) const BLOCK_CONTAINER: &str = "blockContainer";
