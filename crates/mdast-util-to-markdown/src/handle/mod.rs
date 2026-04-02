use std::collections::HashMap;

use crate::types::{HandlerFn, PeekFn};

pub mod blockquote;
pub mod break_;
pub mod code;
pub mod definition;
pub mod delete;
pub mod emphasis;
pub mod heading;
pub mod html;
pub mod image;
pub mod image_reference;
pub mod inline_code;
pub mod link;
pub mod link_reference;
pub mod list;
pub mod list_item;
pub mod paragraph;
pub mod root;
pub mod strong;
pub mod table;
pub mod text;
pub mod thematic_break;

/// Get the default handler registry (CommonMark handlers).
///
/// Port of JS `lib/handle/index.js`.
pub fn default_handlers() -> HashMap<String, HandlerFn> {
    let mut handlers: HashMap<String, HandlerFn> = HashMap::new();

    handlers.insert("root".to_string(), root::handle_root as HandlerFn);
    handlers.insert("text".to_string(), text::handle_text as HandlerFn);
    handlers.insert(
        "paragraph".to_string(),
        paragraph::handle_paragraph as HandlerFn,
    );
    handlers.insert("heading".to_string(), heading::handle_heading as HandlerFn);
    handlers.insert(
        "emphasis".to_string(),
        emphasis::handle_emphasis as HandlerFn,
    );
    handlers.insert("strong".to_string(), strong::handle_strong as HandlerFn);
    handlers.insert(
        "inlineCode".to_string(),
        inline_code::handle_inline_code as HandlerFn,
    );
    handlers.insert("code".to_string(), code::handle_code as HandlerFn);
    handlers.insert("link".to_string(), link::handle_link as HandlerFn);
    handlers.insert("image".to_string(), image::handle_image as HandlerFn);
    handlers.insert("list".to_string(), list::handle_list as HandlerFn);
    handlers.insert(
        "listItem".to_string(),
        list_item::handle_list_item as HandlerFn,
    );
    handlers.insert(
        "blockquote".to_string(),
        blockquote::handle_blockquote as HandlerFn,
    );
    handlers.insert(
        "thematicBreak".to_string(),
        thematic_break::handle_thematic_break as HandlerFn,
    );
    handlers.insert("break".to_string(), break_::handle_break as HandlerFn);
    handlers.insert("html".to_string(), html::handle_html as HandlerFn);
    handlers.insert(
        "definition".to_string(),
        definition::handle_definition as HandlerFn,
    );
    handlers.insert(
        "linkReference".to_string(),
        link_reference::handle_link_reference as HandlerFn,
    );
    handlers.insert(
        "imageReference".to_string(),
        image_reference::handle_image_reference as HandlerFn,
    );

    handlers
}

/// Get the default peek handler registry.
pub fn default_peek_handlers() -> HashMap<String, PeekFn> {
    let mut peek_handlers: HashMap<String, PeekFn> = HashMap::new();

    peek_handlers.insert(
        "emphasis".to_string(),
        emphasis::peek_emphasis as PeekFn,
    );
    peek_handlers.insert("strong".to_string(), strong::peek_strong as PeekFn);
    peek_handlers.insert(
        "inlineCode".to_string(),
        inline_code::peek_inline_code as PeekFn,
    );
    peek_handlers.insert("link".to_string(), link::peek_link as PeekFn);
    peek_handlers.insert("image".to_string(), image::peek_image as PeekFn);
    peek_handlers.insert("html".to_string(), html::peek_html as PeekFn);
    peek_handlers.insert(
        "linkReference".to_string(),
        link_reference::peek_link_reference as PeekFn,
    );
    peek_handlers.insert(
        "imageReference".to_string(),
        image_reference::peek_image_reference as PeekFn,
    );

    peek_handlers
}
