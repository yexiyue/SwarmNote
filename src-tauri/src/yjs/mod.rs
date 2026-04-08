pub mod commands;
pub mod doc_state;
pub mod manager;

use yrs::{Doc, OffsetKind, Options, Transact};

use crate::error::{AppError, AppResult};

/// Y.Doc XML fragment name shared by all modules.
/// Must match the frontend BlockNote editor's fragment name.
pub(crate) const FRAGMENT_NAME: &str = "document-store";

/// Create a temporary Y.Doc with Utf16 offset (matching frontend JS yjs).
///
/// All Y.Docs **must** use `OffsetKind::Utf16` — the yrs default
/// `OffsetKind::Bytes` causes `block_offset` overflow panics on CJK text.
pub(crate) fn create_temp_doc() -> Doc {
    let doc = Doc::with_options(Options {
        offset_kind: OffsetKind::Utf16,
        ..Default::default()
    });
    doc.get_or_insert_xml_fragment(FRAGMENT_NAME);
    doc
}

/// Decode a v1 binary update and apply it to a Y.Doc.
pub(crate) fn apply_update_to_doc(doc: &Doc, data: &[u8], context: &str) -> AppResult<()> {
    let update = yrs::updates::decoder::Decode::decode_v1(data)
        .map_err(|e| AppError::Yjs(format!("decode {context}: {e}")))?;
    doc.transact_mut()
        .apply_update(update)
        .map_err(|e| AppError::Yjs(format!("apply {context}: {e}")))?;
    Ok(())
}

/// Compute blake3 hash of text content for `file_hash` DB column.
pub(crate) fn content_hash(text: &str) -> Vec<u8> {
    blake3::hash(text.as_bytes()).as_bytes().to_vec()
}
