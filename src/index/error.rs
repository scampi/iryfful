//! Errors thrown during indexing.

/// Possible indexing errors.
#[derive(Debug, Fail)]
pub enum IndexingError {
    /// The mapping of a field within the index was already defined.
    #[fail(display = "mapping already exists for field: {}", field)]
    MappingFieldAlreadyExists { field: String },

    /// The mapping for a field does not exist.
    #[fail(display = "missing mapping for field: {}", field)]
    MissingFieldMapping { field: String },
}
