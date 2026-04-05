//! Bidirectional conversion between Markdown, `BlockNote` [`Block`] structs, and yrs `Y.Doc`.
//!
//! Supports GFM Markdown (tables, task lists, strikethrough) and the default `BlockNote` schema.
//!
//! # Core API
//!
//! All conversions go through [`Block`] as the central data type:
//!
//! - **Markdown ↔ Blocks**: [`markdown_to_blocks`] / [`blocks_to_markdown`]
//! - **Y.Doc ↔ Blocks**: [`doc_to_blocks`] / [`blocks_to_doc`]
//! - **Markdown ↔ Y.Doc** (convenience): [`markdown_to_doc`] / [`doc_to_markdown`]
//!
//! # Example
//!
//! ```
//! use yrs_blocknote::{markdown_to_blocks, blocks_to_markdown, BlockType};
//!
//! let blocks = markdown_to_blocks("## Hello **World**\n");
//! assert_eq!(blocks[0].block_type, BlockType::Heading);
//! assert_eq!(blocks[0].props.level, Some(2));
//!
//! let md = blocks_to_markdown(&blocks).unwrap();
//! assert!(md.contains("## Hello"));
//! ```

mod blocks;
mod error;
mod markdown;
mod props;
pub mod schema;
mod yrs_codec;

pub use blocks::{
    Block, BlockContent, InlineContent, Styles, TableCell, TableCellProps, TableCellType,
    TableContent, TableRow,
};
pub use error::{ConvertError, ConvertResult};
pub use props::Props;
pub use schema::BlockType;

use yrs::Doc;

/// Parse GFM Markdown into BlockNote-compatible Blocks.
#[must_use]
pub fn markdown_to_blocks(md: &str) -> Vec<Block> {
    markdown_to_blocks_with(md, default_id_generator)
}

/// Parse GFM Markdown into Blocks with a custom ID generator.
#[must_use]
pub fn markdown_to_blocks_with(md: &str, mut id_gen: impl FnMut() -> String) -> Vec<Block> {
    markdown::parse_markdown(md, &mut id_gen)
}

/// Render Blocks to GFM Markdown string.
///
/// # Errors
///
/// Returns [`ConvertError::MarkdownRender`] if markdown rendering fails.
pub fn blocks_to_markdown(blocks: &[Block]) -> ConvertResult<String> {
    markdown::blocks_to_markdown(blocks)
}

/// Decode a Y.Doc `XmlFragment` into `BlockNote` Blocks.
///
/// # Errors
///
/// Returns [`ConvertError::FragmentNotFound`] if the fragment is empty, or
/// [`ConvertError::InvalidSchema`] if the Y.Doc structure does not match the `BlockNote` schema.
pub fn doc_to_blocks(doc: &Doc, fragment_name: &str) -> ConvertResult<Vec<Block>> {
    yrs_codec::doc_to_blocks(doc, fragment_name)
}

/// Encode `BlockNote` Blocks into a new Y.Doc with the given `XmlFragment` name.
#[must_use]
pub fn blocks_to_doc(blocks: &[Block], fragment_name: &str) -> Doc {
    blocks_to_doc_with(blocks, fragment_name, default_id_generator)
}

/// Encode Blocks into a Y.Doc with a custom ID generator for blocks missing an ID.
#[must_use]
pub fn blocks_to_doc_with(
    blocks: &[Block],
    fragment_name: &str,
    id_gen: impl FnMut() -> String,
) -> Doc {
    yrs_codec::blocks_to_doc(blocks, fragment_name, id_gen)
}

/// Convenience: parse Markdown directly into a Y.Doc.
#[must_use]
pub fn markdown_to_doc(md: &str, fragment_name: &str) -> Doc {
    let blocks = markdown_to_blocks(md);
    blocks_to_doc(&blocks, fragment_name)
}

/// Convenience: export a Y.Doc to Markdown string.
///
/// # Errors
///
/// Returns [`ConvertError::FragmentNotFound`] or [`ConvertError::InvalidSchema`] if the Y.Doc
/// cannot be decoded, or [`ConvertError::MarkdownRender`] if markdown rendering fails.
pub fn doc_to_markdown(doc: &Doc, fragment_name: &str) -> ConvertResult<String> {
    let blocks = doc_to_blocks(doc, fragment_name)?;
    blocks_to_markdown(&blocks)
}

/// Replace the entire content of an existing Y.Doc's `XmlFragment` from Markdown.
///
/// Clears the fragment and re-encodes the parsed blocks in a single transaction.
/// The Doc instance is preserved so CRDT history stays continuous.
pub fn replace_doc_content(doc: &Doc, md: &str, fragment_name: &str) {
    replace_doc_content_with(doc, md, fragment_name, default_id_generator);
}

/// Replace the entire content of an existing Y.Doc's `XmlFragment` from Markdown,
/// with a custom ID generator for blocks missing an ID.
pub fn replace_doc_content_with(
    doc: &Doc,
    md: &str,
    fragment_name: &str,
    id_gen: impl FnMut() -> String,
) {
    let blocks = markdown_to_blocks(md);
    yrs_codec::replace_fragment_content(doc, &blocks, fragment_name, id_gen);
}

/// Default ID generator: uses nanoid, or UUID v7 with the `uuid` feature.
pub fn default_id_generator() -> String {
    #[cfg(feature = "uuid")]
    {
        uuid::Uuid::now_v7().to_string()
    }
    #[cfg(not(feature = "uuid"))]
    {
        nanoid::nanoid!()
    }
}
