use crate::LodError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RdfFormat {
    Turtle,
    NTriples,
    JsonLd,
    RdfXml,
    TriG,
}

impl RdfFormat {
    pub fn parse(value: &str) -> Result<Self, LodError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "ttl" | "turtle" => Ok(Self::Turtle),
            "nt" | "ntriples" | "n-triples" => Ok(Self::NTriples),
            "jsonld" | "json-ld" | "json" => Ok(Self::JsonLd),
            "rdfxml" | "rdf-xml" | "rdf/xml" | "rdf" | "xml" => Ok(Self::RdfXml),
            "trig" => Ok(Self::TriG),
            other => Err(LodError::UnsupportedFormat(other.to_string())),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, LodError> {
        let ext = path.as_ref().extension().and_then(|x| x.to_str()).unwrap_or("");
        match ext.to_ascii_lowercase().as_str() {
            "rdf" | "xml" => Ok(Self::RdfXml),
            "trig" => Ok(Self::TriG),
            other => Self::parse(other),
        }
    }
}
