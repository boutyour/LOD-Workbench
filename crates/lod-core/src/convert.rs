use crate::{parser, ConversionRequest, LodError, RdfFormat};
use std::path::Path;

pub struct ConversionService;

impl ConversionService {
    pub fn convert(&self, req: ConversionRequest) -> Result<(), LodError> {
        // Resolve formats from either the CLI flags or file extensions so the
        // same code path works for manual and implicit conversions.
        let from = match req.source_format.as_deref() {
            Some(s) => Some(RdfFormat::parse(s)?),
            None => Some(RdfFormat::from_path(Path::new(&req.input_path))?),
        };
        let to = match req.target_format.as_deref() {
            Some(s) => Some(RdfFormat::parse(s)?),
            None => Some(RdfFormat::from_path(Path::new(&req.output_path))?),
        };
        let graph = parser::read_graph(&req.input_path, from)?;
        parser::write_graph(&graph, &req.output_path, to)?;
        Ok(())
    }
}
