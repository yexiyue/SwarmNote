use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Block properties, serialized as a flat JSON object with correctly typed values.
///
/// Known `BlockNote` props have typed fields. Unknown or custom props are captured
/// in the `other` map via `#[serde(flatten)]`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Props {
    // ── DefaultProps (shared by paragraph, heading, list items) ──
    /// Background color of the block. Default: `"default"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    /// Text color of the block. Default: `"default"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    /// Text alignment. One of `"left"`, `"center"`, `"right"`, `"justify"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_alignment: Option<String>,

    // ── Heading ──
    /// Heading level (1–6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    /// Whether the heading is toggleable (collapsible).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_toggleable: Option<bool>,

    // ── CheckListItem ──
    /// Whether the check list item is checked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,

    // ── NumberedListItem ──
    /// Start number for ordered lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<usize>,

    // ── CodeBlock ──
    /// Programming language for syntax highlighting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    // ── Image / Video / Audio / File ──
    /// Media name / filename.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Media URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Caption text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// Whether to show the media preview.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_preview: Option<bool>,
    /// Preview width in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_width: Option<u32>,

    // ── Unknown / custom props ──
    /// Any props not covered by the known fields above.
    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

impl Props {
    /// Insert default text props (textColor, backgroundColor, textAlignment).
    pub(crate) fn insert_text_defaults(&mut self) {
        if self.text_color.is_none() {
            self.text_color = Some("default".into());
        }
        if self.background_color.is_none() {
            self.background_color = Some("default".into());
        }
        if self.text_alignment.is_none() {
            self.text_alignment = Some("left".into());
        }
    }

    /// Merge another Props into self (other's values take precedence).
    pub(crate) fn merge_from(&mut self, other: &Props) {
        macro_rules! merge_opt {
            ($field:ident) => {
                if other.$field.is_some() {
                    self.$field = other.$field.clone();
                }
            };
        }
        merge_opt!(background_color);
        merge_opt!(text_color);
        merge_opt!(text_alignment);
        merge_opt!(level);
        merge_opt!(is_toggleable);
        merge_opt!(checked);
        merge_opt!(start);
        merge_opt!(language);
        merge_opt!(name);
        merge_opt!(url);
        merge_opt!(caption);
        merge_opt!(show_preview);
        merge_opt!(preview_width);
        for (k, v) in &other.other {
            self.other.insert(k.clone(), v.clone());
        }
    }
}
