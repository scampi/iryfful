#[derive(Debug, Fail)]
pub enum IndexingError {
    #[fail(display = "mapping already exists for field: {}", field)]
    MappingFieldAlreadyExists { field: String },

    #[fail(display = "missing mapping for field: {}", field)]
    MissingFieldMapping { field: String },
}
