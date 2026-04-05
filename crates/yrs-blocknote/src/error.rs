/// Errors that can occur during format conversion.
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    /// Markdown rendering failed.
    #[error("markdown render error: {0}")]
    MarkdownRender(#[from] std::fmt::Error),
    /// Y.Doc has no `XmlFragment` with the given name.
    #[error("XmlFragment '{0}' not found in Y.Doc")]
    FragmentNotFound(String),
    /// Y.Doc structure doesn't match expected `BlockNote` schema.
    #[error("invalid BlockNote schema: {0}")]
    InvalidSchema(String),
}

/// Result type for conversion operations.
pub type ConvertResult<T> = Result<T, ConvertError>;
