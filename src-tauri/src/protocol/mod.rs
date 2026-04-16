//! Pre-split compatibility shim. All types now live in
//! `swarmnote_core::protocol` — this module re-exports them so existing
//! `use crate::protocol::*` call sites keep compiling during the
//! `extract-swarmnote-core` migration.

pub use swarmnote_core::protocol::*;
