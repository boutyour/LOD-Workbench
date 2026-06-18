use crate::*;

/// Small façade that exposes the core services behind a single entry point.
pub struct LodWorkbench {
    converter: ConversionService,
    inspector: InspectionService,
    validator: ValidationService,
    mapper: MappingService,
    visualizer: VisualizationService,
}

impl Default for LodWorkbench {
    fn default() -> Self {
        Self {
            converter: ConversionService,
            inspector: InspectionService,
            validator: ValidationService,
            mapper: MappingService,
            visualizer: VisualizationService,
        }
    }
}

impl LodWorkbench {
    pub fn convert(&self, req: ConversionRequest) -> Result<(), LodError> {
        self.converter.convert(req)
    }
    pub fn inspect(&self, req: InspectionRequest) -> Result<InspectionReport, LodError> {
        self.inspector.inspect(req)
    }
    pub fn inspect_content(
        &self,
        content: &str,
        format: RdfFormat,
        json_output: Option<String>,
    ) -> Result<InspectionReport, LodError> {
        self.inspector.inspect_content(content, format, json_output)
    }
    pub fn validate(&self, req: ValidationRequest) -> Result<ValidationReport, LodError> {
        self.validator.validate(req)
    }
    pub fn validate_content(
        &self,
        content: &str,
        format: RdfFormat,
        shapes_graph_path: Option<String>,
        report_path: Option<String>,
    ) -> Result<ValidationReport, LodError> {
        self.validator
            .validate_content(content, format, shapes_graph_path, report_path)
    }
    pub fn validate_content_with_format(
        &self,
        content: &str,
        format: RdfFormat,
        shapes_graph_path: Option<String>,
        report_path: Option<String>,
        report_format: Option<ValidationReportFormat>,
    ) -> Result<ValidationReport, LodError> {
        self.validator
            .validate_content_with_format(content, format, shapes_graph_path, report_path, report_format)
    }
    pub fn validate_content_with_shapes(
        &self,
        content: &str,
        format: RdfFormat,
        shapes_content: Option<&str>,
        shapes_format: Option<RdfFormat>,
        report_path: Option<String>,
        report_format: Option<ValidationReportFormat>,
    ) -> Result<ValidationReport, LodError> {
        self.validator.validate_content_with_shapes(
            content,
            format,
            shapes_content,
            shapes_format,
            report_path,
            report_format,
        )
    }
    pub fn validate_content_with_shapes_report(
        &self,
        content: &str,
        format: RdfFormat,
        shapes_content: Option<&str>,
        shapes_format: Option<RdfFormat>,
        report_path: Option<String>,
        report_format: Option<ValidationReportFormat>,
    ) -> Result<ValidationReport, LodError> {
        self.validator.validate_content_with_shapes_report(
            content,
            format,
            shapes_content,
            shapes_format,
            report_path,
            report_format,
        )
    }
    pub fn map_csv_to_rdf(&self, req: MappingRequest) -> Result<(), LodError> {
        self.mapper.map_csv_to_rdf(req)
    }
    pub fn visualize(&self, req: VisualizationRequest) -> Result<(), LodError> {
        self.visualizer.visualize(req)
    }
}
