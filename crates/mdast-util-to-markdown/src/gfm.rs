//! GFM (GitHub Flavored Markdown) extensions for `to_markdown`.
//!
//! Provides extensions for tables, strikethrough, and task list items.

use std::collections::HashMap;

use crate::handle::delete::{handle_delete, peek_delete};
use crate::handle::inline_code::handle_inline_code;
use crate::handle::list_item::handle_list_item;
use crate::handle::table::{handle_table, handle_table_cell, handle_table_row};
use crate::types::{
    ConstructName, Extension, HandlerFn, PeekFn, UnsafePattern,
};

use markdown::mdast::Node;

use crate::state::State;
use crate::types::Info;

/// Options for the GFM table extension.
#[derive(Debug, Clone)]
pub struct GfmTableOptions {
    /// Whether to add a space of padding between delimiters and cells (default: true).
    pub table_cell_padding: bool,
    /// Whether to align the delimiters (default: true).
    pub table_pipe_align: bool,
}

impl Default for GfmTableOptions {
    fn default() -> Self {
        Self {
            table_cell_padding: true,
            table_pipe_align: true,
        }
    }
}

/// Get all GFM extensions.
pub fn gfm() -> Vec<Extension> {
    vec![
        gfm_table(GfmTableOptions::default()),
        gfm_strikethrough(),
        gfm_task_list_item(),
    ]
}

/// Create the GFM table extension.
pub fn gfm_table(_options: GfmTableOptions) -> Extension {
    let mut handlers: HashMap<String, HandlerFn> = HashMap::new();
    handlers.insert(
        "inlineCode".to_string(),
        inline_code_with_table as HandlerFn,
    );
    handlers.insert("table".to_string(), handle_table as HandlerFn);
    handlers.insert("tableCell".to_string(), handle_table_cell as HandlerFn);
    handlers.insert("tableRow".to_string(), handle_table_row as HandlerFn);

    Extension {
        handlers,
        unsafe_patterns: vec![
            UnsafePattern {
                character: '\r',
                before: None,
                after: None,
                at_break: false,
                in_construct: vec![ConstructName::TableCell],
                not_in_construct: vec![],
            },
            UnsafePattern {
                character: '\n',
                before: None,
                after: None,
                at_break: false,
                in_construct: vec![ConstructName::TableCell],
                not_in_construct: vec![],
            },
            // A pipe, when followed by a tab or space (padding), or a dash or colon
            // (unpadded delimiter row), could result in a table.
            UnsafePattern {
                character: '|',
                before: None,
                after: Some("[\\t :-]".to_string()),
                at_break: true,
                in_construct: vec![],
                not_in_construct: vec![],
            },
            // A pipe in a cell must be encoded.
            UnsafePattern {
                character: '|',
                before: None,
                after: None,
                at_break: false,
                in_construct: vec![ConstructName::TableCell],
                not_in_construct: vec![],
            },
            // A colon must be followed by a dash, in which case it could start a
            // delimiter row.
            UnsafePattern {
                character: ':',
                before: None,
                after: Some("-".to_string()),
                at_break: true,
                in_construct: vec![],
                not_in_construct: vec![],
            },
            // A delimiter row can also start with a dash.
            UnsafePattern {
                character: '-',
                before: None,
                after: Some("[:|-]".to_string()),
                at_break: true,
                in_construct: vec![],
                not_in_construct: vec![],
            },
        ],
        join: vec![],
    }
}

/// Create the GFM strikethrough extension.
pub fn gfm_strikethrough() -> Extension {
    let constructs_without_strikethrough = vec![
        ConstructName::Autolink,
        ConstructName::DestinationLiteral,
        ConstructName::DestinationRaw,
        ConstructName::Reference,
        ConstructName::TitleQuote,
        ConstructName::TitleApostrophe,
    ];

    let mut handlers: HashMap<String, HandlerFn> = HashMap::new();
    handlers.insert("delete".to_string(), handle_delete as HandlerFn);

    Extension {
        handlers,
        unsafe_patterns: vec![UnsafePattern {
            character: '~',
            before: None,
            after: None,
            at_break: false,
            in_construct: vec![ConstructName::Phrasing],
            not_in_construct: constructs_without_strikethrough,
        }],
        join: vec![],
    }
}

/// Create the GFM task list item extension.
pub fn gfm_task_list_item() -> Extension {
    let mut handlers: HashMap<String, HandlerFn> = HashMap::new();
    handlers.insert(
        "listItem".to_string(),
        list_item_with_task as HandlerFn,
    );

    Extension {
        handlers,
        unsafe_patterns: vec![UnsafePattern {
            character: '-',
            before: None,
            after: Some("[:|-]".to_string()),
            at_break: true,
            in_construct: vec![],
            not_in_construct: vec![],
        }],
        join: vec![],
    }
}

/// Get GFM peek handlers (for delete).
pub fn gfm_peek_handlers() -> HashMap<String, PeekFn> {
    let mut peek_handlers: HashMap<String, PeekFn> = HashMap::new();
    peek_handlers.insert("delete".to_string(), peek_delete as PeekFn);
    peek_handlers
}

/// Inline code handler that escapes pipes when in a table cell.
fn inline_code_with_table(
    node: &Node,
    parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    let mut value = handle_inline_code(node, parent, state, info);

    if state.stack.contains(&ConstructName::TableCell) {
        value = value.replace('|', "\\|");
    }

    value
}

/// List item handler with task list support.
///
/// Checks `ListItem.checked` and delegates to the default `handle_list_item`.
fn list_item_with_task(
    node: &Node,
    parent: Option<&Node>,
    state: &mut State,
    info: &Info,
) -> String {
    if let Node::ListItem(list_item) = node {
        let head = list_item.children.first();
        let checkable = list_item.checked.is_some()
            && head.is_some()
            && matches!(head.unwrap(), Node::Paragraph(_));

        let checkbox = if checkable {
            if list_item.checked == Some(true) {
                "[x] "
            } else {
                "[ ] "
            }
        } else {
            ""
        };

        let mut tracker = state.create_tracker(info);

        if checkable {
            tracker.r#move(checkbox);
        }

        let adjusted_info = Info {
            line: tracker.current().line,
            column: tracker.current().column,
            line_shift: tracker.current().line_shift,
            before: info.before.clone(),
            after: info.after.clone(),
        };

        let mut value = handle_list_item(node, parent, state, &adjusted_info);

        if checkable {
            // Insert checkbox after the bullet marker
            let re = regex::Regex::new(r"^(?:[*+\-]|\d+\.)([\r\n]| {1,3})").unwrap();
            if let Some(mat) = re.find(&value) {
                let matched = mat.as_str();
                let rest = &value[mat.end()..];
                value = format!("{}{}{}", matched, checkbox, rest);
            }
        }

        value
    } else {
        handle_list_item(node, parent, state, info)
    }
}
