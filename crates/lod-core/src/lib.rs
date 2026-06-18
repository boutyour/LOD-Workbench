//! **LOD Core** — The shared domain and services for the LOD Workbench toolkit.
//!
//! This crate provides the core data model ([`LodGraph`], [`Triple`], [`Node`]),
//! RDF parsing and serialization for Turtle, N-Triples, RDF/XML, TriG, and a
//! compact JSON-LD subset, plus service modules for conversion, inspection,
//! validation, CSV→RDF mapping, and HTML graph visualization.

pub mod convert;
pub mod error;
pub mod facade;
pub mod format;
pub mod inspect;
pub mod map;
pub mod model;
pub mod parser;
pub mod validate;
pub mod visualize;

pub use convert::*;
pub use error::LodError;
pub use facade::LodWorkbench;
pub use format::RdfFormat;
pub use inspect::*;
pub use map::*;
pub use model::*;
pub use validate::*;
pub use visualize::*;

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn lod_workbench_default_constructs() {
        let wb = LodWorkbench::default();
        // Validate with no shapes — should be conformant for empty input
        let req = ValidationRequest {
            data_graph_path: String::new(),
            shapes_graph_path: None,
            report_path: None,
            report_format: None,
        };
        // Empty path will error with I/O, which is fine — we tested construction.
        let _ = wb.validate(req);
    }

    #[test]
    fn inspection_service_errors_on_missing_file() {
        let svc = InspectionService;
        let req = InspectionRequest {
            input_path: "/nonexistent/file.ttl".into(),
            input_format: Some("turtle".into()),
            json_output: None,
        };
        let result = svc.inspect(req);
        assert!(result.is_err());
    }

    #[test]
    fn validation_service_errors_on_missing_file() {
        let svc = ValidationService;
        let req = ValidationRequest {
            data_graph_path: "/nonexistent/file.ttl".into(),
            shapes_graph_path: None,
            report_path: None,
            report_format: None,
        };
        let result = svc.validate(req);
        assert!(result.is_err());
    }

    #[test]
    fn visualization_service_errors_on_missing_file() {
        let svc = VisualizationService;
        let req = VisualizationRequest {
            input_path: "/nonexistent/file.ttl".into(),
            input_format: Some("turtle".into()),
            output_path: "/tmp/out.html".into(),
        };
        let result = svc.visualize(req);
        assert!(result.is_err());
    }

    #[test]
    fn format_from_path_extension() {
        assert_eq!(RdfFormat::from_path("data.ttl").unwrap(), RdfFormat::Turtle);
        assert_eq!(RdfFormat::from_path("data.nt").unwrap(), RdfFormat::NTriples);
        assert_eq!(RdfFormat::from_path("data.jsonld").unwrap(), RdfFormat::JsonLd);
        assert_eq!(RdfFormat::from_path("data.rdf").unwrap(), RdfFormat::RdfXml);
        assert_eq!(RdfFormat::from_path("data.xml").unwrap(), RdfFormat::RdfXml);
        assert_eq!(RdfFormat::from_path("data.trig").unwrap(), RdfFormat::TriG);
        assert!(RdfFormat::from_path("data.xyz").is_err());
    }

    #[test]
    fn conversion_service_rejects_bad_format() {
        let svc = ConversionService;
        let req = ConversionRequest {
            input_path: "in.ttl".into(),
            output_path: "out.nt".into(),
            source_format: Some("bad-format".into()),
            target_format: None,
        };
        let result = svc.convert(req);
        assert!(result.is_err());
    }

    #[test]
    fn mapping_service_rejects_missing_files() {
        let svc = MappingService;
        let req = MappingRequest {
            input_path: "/nonexistent.csv".into(),
            mapping_path: "/nonexistent.yml".into(),
            output_path: "/tmp/out.ttl".into(),
            output_format: Some("turtle".into()),
        };
        let result = svc.map_csv_to_rdf(req);
        assert!(result.is_err());
    }

    #[test]
    fn format_parse_accepts_new_formats() {
        assert_eq!(RdfFormat::parse("rdf/xml").unwrap(), RdfFormat::RdfXml);
        assert_eq!(RdfFormat::parse("rdfxml").unwrap(), RdfFormat::RdfXml);
        assert_eq!(RdfFormat::parse("trig").unwrap(), RdfFormat::TriG);
    }
}
