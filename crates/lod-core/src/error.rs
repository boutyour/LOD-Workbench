use thiserror::Error;

#[derive(Debug, Error)]
pub enum LodError {
    #[error("unsupported RDF format: {0}")]
    UnsupportedFormat(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("RDF parsing error: {0}")]
    RdfParsing(String),
    #[error("mapping error: {0}")]
    Mapping(String),
    #[error("validation error: {0}")]
    Validation(String),
}
