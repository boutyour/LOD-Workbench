use crate::{
    parser, LodError, LodGraph, Node, RdfFormat, ValidationIssue, ValidationReport, ValidationReportFormat,
    ValidationRequest,
};
#[cfg(feature = "rudof-shacl")]
use rudof_lib::{
    formats::{
        DataFormat as RudofDataFormat, InputSpec, ResultShaclValidationFormat, ShaclFormat, ShaclValidationMode,
    },
    Rudof, RudofConfig,
};
use std::fs;

pub struct ValidationService;

impl ValidationService {
    pub fn validate(&self, req: ValidationRequest) -> Result<ValidationReport, LodError> {
        let content = fs::read_to_string(&req.data_graph_path)?;
        let fmt = RdfFormat::from_path(&req.data_graph_path)?;
        self.validate_content_with_format(&content, fmt, req.shapes_graph_path, req.report_path, req.report_format)
    }

    pub fn validate_content(
        &self,
        content: &str,
        fmt: RdfFormat,
        shapes_graph_path: Option<String>,
        report_path: Option<String>,
    ) -> Result<ValidationReport, LodError> {
        self.validate_content_with_format(content, fmt, shapes_graph_path, report_path, None)
    }

    pub fn validate_content_with_format(
        &self,
        content: &str,
        fmt: RdfFormat,
        shapes_graph_path: Option<String>,
        report_path: Option<String>,
        report_format: Option<ValidationReportFormat>,
    ) -> Result<ValidationReport, LodError> {
        self.validate_content_internal(
            content,
            fmt,
            shapes_graph_path.as_deref().map(ShapesInput::Path),
            report_path,
            report_format,
            false,
        )
    }

    pub fn validate_content_with_shapes(
        &self,
        content: &str,
        fmt: RdfFormat,
        shapes_content: Option<&str>,
        shapes_format: Option<RdfFormat>,
        report_path: Option<String>,
        report_format: Option<ValidationReportFormat>,
    ) -> Result<ValidationReport, LodError> {
        let shapes = shapes_content.map(|content| ShapesInput::Content {
            content,
            format: shapes_format.unwrap_or(RdfFormat::Turtle),
        });
        self.validate_content_internal(content, fmt, shapes, report_path, report_format, false)
    }

    pub fn validate_content_with_shapes_report(
        &self,
        content: &str,
        fmt: RdfFormat,
        shapes_content: Option<&str>,
        shapes_format: Option<RdfFormat>,
        report_path: Option<String>,
        report_format: Option<ValidationReportFormat>,
    ) -> Result<ValidationReport, LodError> {
        let shapes = shapes_content.map(|content| ShapesInput::Content {
            content,
            format: shapes_format.unwrap_or(RdfFormat::Turtle),
        });
        self.validate_content_internal(content, fmt, shapes, report_path, report_format, true)
    }

    fn validate_content_internal(
        &self,
        content: &str,
        fmt: RdfFormat,
        shapes: Option<ShapesInput<'_>>,
        report_path: Option<String>,
        report_format: Option<ValidationReportFormat>,
        run_shacl: bool,
    ) -> Result<ValidationReport, LodError> {
        let mut issues = Vec::new();
        // Validation is split in two passes:
        // - syntax and IRI hygiene for the input graph and optional shapes graph
        // - optional SHACL constraint validation in the dedicated SHACL tab
        match parser::parse_graph(content, fmt) {
            Ok(graph) => validate_graph(&graph, &mut issues, "input"),
            Err(e) => issues.push(parse_issue_from_error(content, e.to_string(), "input")),
        }
        if run_shacl && shapes.is_none() {
            issues.push(ValidationIssue {
                severity: "Info".into(),
                message: "No SHACL shapes were provided, so SHACL validation was skipped.".into(),
                line: None,
                column: None,
                token: None,
                source: Some("shacl".into()),
                suggestion: Some(
                    "Load or paste a SHACL shapes graph in the Shapes panel to run constraint validation.".into(),
                ),
                ..Default::default()
            });
        }
        if let Some(shapes) = shapes {
            let (shapes_content, shapes_format) = match shapes {
                ShapesInput::Path(path) => (fs::read_to_string(path)?, RdfFormat::from_path(path)?),
                ShapesInput::Content { content, format } => (content.to_string(), format),
            };
            match parser::parse_graph(&shapes_content, shapes_format) {
                Ok(graph) => validate_graph(&graph, &mut issues, "shapes"),
                Err(e) => issues.push(parse_issue_from_error(&shapes_content, e.to_string(), "shapes")),
            }
            if run_shacl {
                #[cfg(feature = "rudof-shacl")]
                run_shacl_validation(content, fmt, &shapes_content, shapes_format, &mut issues)?;
                #[cfg(not(feature = "rudof-shacl"))]
                note_shacl_feature_disabled(shapes, &mut issues);
            }
        }
        let conforms = !issues.iter().any(|i| i.severity == "Violation");
        let report = ValidationReport { conforms, issues };
        if let Some(path) = report_path {
            write_report(&report, &path, report_format)?;
        }
        Ok(report)
    }
}

#[derive(Clone, Copy)]
enum ShapesInput<'a> {
    Path(&'a str),
    Content { content: &'a str, format: RdfFormat },
}

impl ShapesInput<'_> {
    #[cfg(not(feature = "rudof-shacl"))]
    fn label(&self) -> String {
        match self {
            Self::Path(path) => format!("`{path}`"),
            Self::Content { content, format } => {
                format!("inline SHACL shapes ({format:?}, {} bytes)", content.len())
            }
        }
    }
}

#[cfg(not(feature = "rudof-shacl"))]
fn note_shacl_feature_disabled(shapes: ShapesInput<'_>, issues: &mut Vec<ValidationIssue>) {
    issues.push(ValidationIssue {
        severity: "Info".into(),
        message: format!(
            "SHACL shapes were provided as {}, but real SHACL validation requires building with the `rudof-shacl` feature.",
            shapes.label()
        ),
        line: None,
        column: None,
        token: None,
        source: Some("shacl".into()),
        suggestion: Some("Run with `cargo run -p lod --features lod-core/rudof-shacl -- validate ...` to enable Rudof SHACL validation.".into()),
        ..Default::default()
    });
}

#[cfg(feature = "rudof-shacl")]
fn run_shacl_validation(
    content: &str,
    fmt: RdfFormat,
    shapes_content: &str,
    shapes_format: RdfFormat,
    issues: &mut Vec<ValidationIssue>,
) -> Result<(), LodError> {
    let data_format = to_rudof_data_format(fmt);
    let shacl_format = to_rudof_shacl_format(shapes_format);

    let data_input = InputSpec::str(content);
    let data_inputs = [data_input];
    let shapes_input = InputSpec::str(&shapes_content);
    let mode = ShaclValidationMode::Native;
    // Use the detailed SHACL table so we can surface the failing focus node,
    // constraint component, path, and source shape in the UI.
    let result_format = ResultShaclValidationFormat::Details;
    let mut rudof = Rudof::new(RudofConfig::default());

    rudof
        .load_data()
        .with_data(&data_inputs)
        .with_data_format(&data_format)
        .execute()
        .map_err(|e| LodError::Validation(format!("SHACL data graph loading failed: {e}")))?;

    rudof
        .load_shacl_shapes()
        .with_shacl_schema(&shapes_input)
        .with_shacl_schema_format(&shacl_format)
        .execute()
        .map_err(|e| LodError::Validation(format!("SHACL shapes loading failed: {e}")))?;

    rudof
        .validate_shacl()
        .with_shacl_validation_mode(&mode)
        .execute()
        .map_err(|e| LodError::Validation(format!("SHACL validation failed: {e}")))?;

    let mut buffer = Vec::new();
    rudof
        .serialize_shacl_validation_results(&mut buffer)
        .with_result_shacl_validation_format(&result_format)
        .execute()
        .map_err(|e| LodError::Validation(format!("SHACL report serialization failed: {e}")))?;
    let report = String::from_utf8_lossy(&buffer).trim().to_string();

    if is_clean_shacl_report(&report) {
        return Ok(());
    }

    if report != "Conforms" {
        let mut parsed = parse_shacl_validation_details(&report);
        if parsed.is_empty() {
            issues.push(ValidationIssue {
                severity: "Violation".into(),
                message: format!("SHACL validation failed:\n{report}"),
                line: None,
                column: None,
                token: None,
                source: Some("shacl".into()),
                suggestion: Some("Inspect the SHACL validation report for focus nodes and constraint messages.".into()),
                details: Some(report),
                ..Default::default()
            });
        } else {
            issues.append(&mut parsed);
        }
    }
    Ok(())
}

#[cfg(feature = "rudof-shacl")]
fn is_clean_shacl_report(report: &str) -> bool {
    let normalized = report.trim();
    normalized.eq_ignore_ascii_case("Conforms")
        || normalized.eq_ignore_ascii_case("No Errors found")
        || normalized.eq_ignore_ascii_case("No errors found")
}

#[cfg(feature = "rudof-shacl")]
#[derive(Debug, Default, Clone)]
struct ShaclDetailRow {
    severity: String,
    node: String,
    component: String,
    path: String,
    value: String,
    source_shape: String,
    details: String,
}

#[cfg(feature = "rudof-shacl")]
fn parse_shacl_validation_details(report: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut current: Option<ShaclDetailRow> = None;

    for line in report.lines() {
        let cleaned = strip_ansi_escape_sequences(line);
        let trimmed = cleaned.trim();
        if trimmed.is_empty() || is_shacl_border_line(trimmed) || is_shacl_header_line(trimmed) {
            continue;
        }
        let Some(cells) = split_shacl_table_row(&cleaned) else {
            continue;
        };

        if cells.is_empty() || cells.iter().all(|c| c.is_empty()) {
            continue;
        }

        let severity = cells.get(0).map(|s| s.trim()).unwrap_or("");
        if !severity.is_empty() {
            if let Some(prev) = current.take() {
                issues.push(shacl_detail_row_to_issue(prev));
            }
            current = Some(ShaclDetailRow {
                severity: severity.to_string(),
                node: cell_at(&cells, 1),
                component: cell_at(&cells, 2),
                path: cell_at(&cells, 3),
                value: cell_at(&cells, 4),
                source_shape: cell_at(&cells, 5),
                details: cell_at(&cells, 6),
            });
        } else if let Some(row) = current.as_mut() {
            merge_detail_cell(&mut row.node, cell_at(&cells, 1));
            merge_detail_cell(&mut row.component, cell_at(&cells, 2));
            merge_detail_cell(&mut row.path, cell_at(&cells, 3));
            merge_detail_cell(&mut row.value, cell_at(&cells, 4));
            merge_detail_cell(&mut row.source_shape, cell_at(&cells, 5));
            merge_detail_cell(&mut row.details, cell_at(&cells, 6));
        }
    }

    if let Some(last) = current.take() {
        issues.push(shacl_detail_row_to_issue(last));
    }

    issues
}

#[cfg(feature = "rudof-shacl")]
fn split_shacl_table_row(line: &str) -> Option<Vec<String>> {
    if !line.contains('│') {
        return None;
    }

    let mut cells: Vec<String> = line.split('│').map(|cell| cell.trim().to_string()).collect();
    if matches!(cells.first(), Some(cell) if cell.is_empty()) {
        cells.remove(0);
    }
    if matches!(cells.last(), Some(cell) if cell.is_empty()) {
        cells.pop();
    }

    Some(cells)
}

fn strip_ansi_escape_sequences(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                let _ = chars.next();
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
                continue;
            }
        }
        out.push(ch);
    }

    out
}

#[cfg(feature = "rudof-shacl")]
fn is_shacl_header_line(line: &str) -> bool {
    line.contains("Severity") && line.contains("Node") && line.contains("Component") && line.contains("Source shape")
}

#[cfg(feature = "rudof-shacl")]
fn is_shacl_border_line(line: &str) -> bool {
    line.chars().all(|ch| {
        matches!(
            ch,
            '─' | '═'
                | '│'
                | '┌'
                | '┐'
                | '└'
                | '┘'
                | '┬'
                | '┴'
                | '┼'
                | '├'
                | '┤'
                | '╭'
                | '╮'
                | '╰'
                | '╯'
                | ' '
                | '━'
                | '┃'
                | '┏'
                | '┓'
                | '┗'
                | '┛'
                | '┣'
                | '┫'
                | '┳'
                | '┻'
                | '╋'
        )
    })
}

#[cfg(feature = "rudof-shacl")]
fn cell_at(cells: &[String], idx: usize) -> String {
    cells
        .get(idx)
        .map(|s| strip_ansi_escape_sequences(s).trim().to_string())
        .unwrap_or_default()
}

#[cfg(feature = "rudof-shacl")]
fn merge_detail_cell(current: &mut String, next: String) {
    if next.is_empty() {
        return;
    }
    if current.is_empty() {
        *current = next;
    } else if !current.contains(&next) {
        if current.ends_with('\n') {
            current.push_str(&next);
        } else {
            current.push_str(" | ");
            current.push_str(&next);
        }
    }
}

#[cfg(feature = "rudof-shacl")]
fn shacl_detail_row_to_issue(row: ShaclDetailRow) -> ValidationIssue {
    let severity = normalize_shacl_severity(&row.severity);
    let summary = shacl_issue_summary(&row);
    ValidationIssue {
        severity,
        message: summary,
        line: None,
        column: None,
        token: non_empty(row.node.clone()).or_else(|| non_empty(row.value.clone())),
        suggestion: (!row.details.is_empty()).then(|| row.details.clone()),
        source: Some("shacl".into()),
        focus_node: non_empty(row.node),
        constraint_component: non_empty(row.component),
        path: non_empty(row.path),
        value: non_empty(row.value),
        source_shape: non_empty(row.source_shape),
        details: non_empty(row.details),
    }
}

#[cfg(feature = "rudof-shacl")]
fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

#[cfg(feature = "rudof-shacl")]
fn shacl_issue_summary(row: &ShaclDetailRow) -> String {
    let node = if row.node.is_empty() {
        "Unknown focus node"
    } else {
        &row.node
    };
    let component = if row.component.is_empty() {
        "constraint"
    } else {
        &row.component
    };
    if row.path.is_empty() {
        format!("{node} failed {component}")
    } else {
        format!("{node} failed {component} on {}", row.path)
    }
}

#[cfg(feature = "rudof-shacl")]
fn normalize_shacl_severity(severity: &str) -> String {
    let short = severity
        .rsplit(['#', '/'])
        .next()
        .unwrap_or(severity)
        .trim_matches(|c| c == '<' || c == '>')
        .trim();
    match short.to_ascii_lowercase().as_str() {
        "violation" => "Violation".into(),
        "warning" => "Warning".into(),
        "info" => "Info".into(),
        other if other.is_empty() => "Violation".into(),
        other => other.to_string(),
    }
}

#[cfg(feature = "rudof-shacl")]
fn to_rudof_data_format(format: RdfFormat) -> RudofDataFormat {
    match format {
        RdfFormat::Turtle => RudofDataFormat::Turtle,
        RdfFormat::NTriples => RudofDataFormat::NTriples,
        RdfFormat::JsonLd => RudofDataFormat::JsonLd,
    }
}

#[cfg(feature = "rudof-shacl")]
fn to_rudof_shacl_format(format: RdfFormat) -> ShaclFormat {
    match format {
        RdfFormat::Turtle => ShaclFormat::Turtle,
        RdfFormat::NTriples => ShaclFormat::NTriples,
        RdfFormat::JsonLd => ShaclFormat::JsonLd,
    }
}

fn is_http_iri(iri: &str) -> bool {
    iri.starts_with("http://") || iri.starts_with("https://")
}

fn validate_graph(graph: &LodGraph, issues: &mut Vec<ValidationIssue>, source: &str) {
    let mut seen = std::collections::BTreeSet::new();
    for (idx, t) in graph.triples.iter().enumerate() {
        // Each triple is checked independently so errors can be reported with
        // a simple line number that is useful in the UI.
        if !seen.insert(t.clone()) {
            issues.push(ValidationIssue {
                severity: "Info".into(),
                message: "Duplicate triple encountered; serializers normalize repeated triples.".into(),
                line: Some(idx + 1),
                column: None,
                token: None,
                suggestion: Some("Remove repeated triples to keep the graph compact.".into()),
                source: Some(source.to_string()),
                ..Default::default()
            });
        }
        if !is_http_iri(&t.predicate) {
            issues.push(ValidationIssue {
                severity: "Warning".into(),
                message: format!("Predicate is not an HTTP IRI: {}", t.predicate),
                line: Some(idx + 1),
                column: None,
                token: Some(t.predicate.clone()),
                suggestion: Some("Use an absolute HTTP or HTTPS predicate IRI when possible.".into()),
                source: Some(source.to_string()),
                ..Default::default()
            });
        }
        if let Node::Iri(iri) = &t.subject {
            check_iri(iri, idx + 1, issues, source);
        }
        if let Node::Iri(iri) = &t.object {
            check_iri(iri, idx + 1, issues, source);
        }
    }
}

fn check_iri(iri: &str, line: usize, issues: &mut Vec<ValidationIssue>, source: &str) {
    if iri.contains(' ') {
        issues.push(ValidationIssue {
            severity: "Violation".into(),
            message: format!("IRI contains spaces: {iri}"),
            line: Some(line),
            column: None,
            token: Some(iri.to_string()),
            suggestion: Some("Remove whitespace or encode the resource with a valid IRI.".into()),
            source: Some(source.to_string()),
            ..Default::default()
        });
    }
    if !is_http_iri(iri) && !iri.starts_with("_:") {
        issues.push(ValidationIssue {
            severity: "Warning".into(),
            message: format!("IRI is not dereferenceable HTTP(S): {iri}"),
            line: Some(line),
            column: None,
            token: Some(iri.to_string()),
            suggestion: Some("This is allowed in RDF, but HTTP(S) IRIs are easier to reuse.".into()),
            source: Some(source.to_string()),
            ..Default::default()
        });
    }
}

fn parse_issue_from_error(content: &str, error: String, source: &str) -> ValidationIssue {
    let (line, message) = split_line_prefix(&error);
    let line_text = line
        .and_then(|n| content.lines().nth(n.saturating_sub(1)))
        .unwrap_or("");
    let (column, token, suggestion) = infer_parse_details(&message, line_text);
    ValidationIssue {
        severity: "Violation".into(),
        message,
        line,
        column,
        token,
        suggestion,
        source: Some(source.to_string()),
        ..Default::default()
    }
}

fn split_line_prefix(error: &str) -> (Option<usize>, String) {
    let Some(rest) = error.strip_prefix("line ") else {
        return (None, error.to_string());
    };
    let Some((line_part, message)) = rest.split_once(':') else {
        return (None, error.to_string());
    };
    let line = line_part.trim().parse::<usize>().ok();
    (line, message.trim().to_string())
}

fn infer_parse_details(message: &str, line_text: &str) -> (Option<usize>, Option<String>, Option<String>) {
    let token = if let Some(prefix) = message.strip_prefix("unknown prefix `") {
        prefix.split('`').next().map(|p| format!("{p}:"))
    } else if let Some(term) = message.strip_prefix("cannot expand term `") {
        term.split('`').next().map(ToString::to_string)
    } else if message.contains("missing `.` terminator") {
        Some(".".into())
    } else if message.contains("unterminated literal") {
        Some("\"".into())
    } else if message.contains("invalid prefix IRI") || message.contains("invalid base IRI") {
        Some("<iri>".into())
    } else {
        None
    };

    let column = token
        .as_deref()
        .and_then(|tok| find_column(line_text, tok))
        .or_else(|| {
            if message.contains("missing `.` terminator") {
                Some(line_text.trim_end().len().saturating_add(1))
            } else if message.contains("unterminated literal") {
                line_text.find('"').map(|c| c + 1)
            } else {
                None
            }
        });

    let suggestion = if message.contains("unknown prefix") {
        Some("Add a matching @prefix declaration or fix the prefix name.".into())
    } else if message.contains("missing `.` terminator") {
        Some("Terminate the declaration with a trailing .".into())
    } else if message.contains("unterminated literal") {
        Some("Close the literal with a matching \" quote.".into())
    } else if message.contains("invalid prefix IRI") || message.contains("invalid base IRI") {
        Some("Wrap the IRI in <...> and ensure it is a valid absolute or relative IRI.".into())
    } else if message.contains("unexpected whitespace in object") {
        Some("Wrap IRIs in <...> or use a single valid RDF term.".into())
    } else {
        None
    };

    (column, token, suggestion)
}

fn find_column(line_text: &str, token: &str) -> Option<usize> {
    line_text.find(token).map(|idx| idx + 1).or_else(|| {
        line_text
            .trim_start()
            .find(token)
            .map(|idx| line_text.len() - line_text.trim_start().len() + idx + 1)
    })
}

fn infer_report_format(path: &str) -> Option<ValidationReportFormat> {
    if path.ends_with(".json") {
        Some(ValidationReportFormat::Json)
    } else if path.ends_with(".txt") || path.ends_with(".text") {
        Some(ValidationReportFormat::Text)
    } else if path.ends_with(".html") || path.ends_with(".htm") {
        Some(ValidationReportFormat::Html)
    } else {
        None
    }
}

fn write_report(
    report: &ValidationReport,
    path: &str,
    report_format: Option<ValidationReportFormat>,
) -> Result<(), LodError> {
    match report_format
        .or_else(|| infer_report_format(path))
        .unwrap_or(ValidationReportFormat::Html)
    {
        ValidationReportFormat::Json => fs::write(path, serde_json::to_string_pretty(report)?)?,
        ValidationReportFormat::Text => fs::write(path, validation_text(report))?,
        ValidationReportFormat::Html => fs::write(path, validation_html(report))?,
    }
    Ok(())
}

pub fn render_validation_report(report: &ValidationReport, format: ValidationReportFormat) -> String {
    match format {
        ValidationReportFormat::Html => validation_html(report),
        ValidationReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ValidationReportFormat::Text => validation_text(report),
    }
}

fn validation_text(report: &ValidationReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("Conforms: {}\n", report.conforms));
    out.push_str(&format!("Issues: {}\n", report.issues.len()));
    for issue in &report.issues {
        out.push_str(&format!(
            "- [{}] {}{}{}\n",
            issue.severity,
            issue.message,
            issue.line.map(|l| format!(" (line {l})")).unwrap_or_default(),
            issue.column.map(|c| format!(":{c}")).unwrap_or_default()
        ));
        if let Some(source) = &issue.source {
            out.push_str(&format!("  source: {source}\n"));
        }
        if let Some(node) = &issue.focus_node {
            out.push_str(&format!("  node: {node}\n"));
        }
        if let Some(component) = &issue.constraint_component {
            out.push_str(&format!("  constraint: {component}\n"));
        }
        if let Some(path) = &issue.path {
            out.push_str(&format!("  path: {path}\n"));
        }
        if let Some(value) = &issue.value {
            out.push_str(&format!("  value: {value}\n"));
        }
        if let Some(source_shape) = &issue.source_shape {
            out.push_str(&format!("  source shape: {source_shape}\n"));
        }
        if let Some(token) = &issue.token {
            out.push_str(&format!("  token: {token}\n"));
        }
        if let Some(suggestion) = &issue.suggestion {
            out.push_str(&format!("  hint: {suggestion}\n"));
        }
        if let Some(details) = &issue.details {
            out.push_str(&format!("  details: {details}\n"));
        }
    }
    out
}

fn validation_html(report: &ValidationReport) -> String {
    let issues = report.issues.len();
    let violations = report
        .issues
        .iter()
        .filter(|i| i.severity.eq_ignore_ascii_case("Violation"))
        .count();
    let warnings = report
        .issues
        .iter()
        .filter(|i| i.severity.eq_ignore_ascii_case("Warning"))
        .count();
    let infos = report
        .issues
        .iter()
        .filter(|i| i.severity.eq_ignore_ascii_case("Info"))
        .count();
    let rows = if report.issues.is_empty() {
        String::from(r#"<tr><td colspan="7" class="empty">No validation issues were found.</td></tr>"#)
    } else {
        let mut rows = String::new();
        for i in &report.issues {
            rows.push_str(&format!(
                "<tr><td><span class=\"sev sev-{}\">{}</span></td><td><span class=\"source-chip {}\">{}</span></td><td>{}</td><td>{}</td><td>{}</td><td><div class=\"issue-message\">{}</div></td><td>{}</td></tr>",
                i.severity.to_ascii_lowercase(),
                i.severity,
                i.source.as_deref().unwrap_or("input").to_ascii_lowercase(),
                escape(i.source.as_deref().unwrap_or("input")),
                i.line.map(|x| x.to_string()).unwrap_or_else(|| "—".into()),
                i.column.map(|x| x.to_string()).unwrap_or_else(|| "—".into()),
                i.token.as_deref().map(escape).unwrap_or_else(|| "—".into()),
                escape(&i.message),
                i.suggestion.as_deref().map(escape).unwrap_or_else(|| "—".into())
            ));
        }
        rows
    };
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>LOD Validation Report</title><style>
            :root {{ color-scheme: light; --bg: #f8fafc; --surface: #ffffff; --border: #e2e8f0; --text: #0f172a; --muted: #64748b; --ok: #166534; --ok-bg: #ecfdf5; --warn: #92400e; --warn-bg: #fffbeb; --bad: #991b1b; --bad-bg: #fef2f2; }}
            * {{ box-sizing: border-box; }}
            body {{ margin: 0; font-family: system-ui, -apple-system, Segoe UI, sans-serif; background: linear-gradient(180deg, #eef2ff 0%, var(--bg) 28%); color: var(--text); }}
            .wrap {{ max-width: 1100px; margin: 0 auto; padding: 2rem 1rem 3rem; }}
            .hero {{ background: var(--surface); border: 1px solid var(--border); border-radius: 20px; padding: 1.4rem 1.5rem; box-shadow: 0 10px 30px rgba(15, 23, 42, 0.06); }}
            .eyebrow {{ color: var(--muted); font-size: .82rem; text-transform: uppercase; letter-spacing: .08em; margin-bottom: .35rem; }}
            h1 {{ margin: 0; font-size: 2rem; line-height: 1.15; }}
            .sub {{ margin-top: .35rem; color: var(--muted); }}
            .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: .75rem; margin: 1rem 0 1.25rem; }}
            .card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 16px; padding: .9rem 1rem; }}
            .card .k {{ font-size: .72rem; text-transform: uppercase; letter-spacing: .08em; color: var(--muted); }}
            .card .v {{ font-size: 1.7rem; font-weight: 800; margin-top: .2rem; }}
            .ok {{ color: var(--ok); background: var(--ok-bg); }}
            .warn {{ color: var(--warn); background: var(--warn-bg); }}
            .bad {{ color: var(--bad); background: var(--bad-bg); }}
            .panel {{ background: var(--surface); border: 1px solid var(--border); border-radius: 20px; overflow: hidden; box-shadow: 0 10px 30px rgba(15, 23, 42, 0.06); }}
            table {{ width: 100%; border-collapse: collapse; }}
            th, td {{ padding: .9rem 1rem; text-align: left; border-bottom: 1px solid var(--border); vertical-align: top; }}
            th {{ position: sticky; top: 0; background: rgba(248, 250, 252, .95); backdrop-filter: blur(8px); font-size: .8rem; color: var(--muted); text-transform: uppercase; letter-spacing: .06em; }}
            tr:last-child td {{ border-bottom: none; }}
            .sev {{ display: inline-flex; align-items: center; padding: .2rem .55rem; border-radius: 999px; font-size: .72rem; font-weight: 700; text-transform: uppercase; letter-spacing: .04em; }}
            .sev-violation {{ background: var(--bad-bg); color: var(--bad); }}
            .sev-warning {{ background: var(--warn-bg); color: var(--warn); }}
            .sev-info {{ background: #eff6ff; color: #1d4ed8; }}
            .issue-message {{ font-weight: 600; }}
            .issue-details {{ color: var(--muted); margin-top: .25rem; white-space: pre-wrap; }}
            .empty {{ text-align: center; color: var(--muted); padding: 1.5rem 1rem; }}
            .footer {{ margin-top: .85rem; color: var(--muted); font-size: .85rem; }}
            .source-chip {{ display: inline-flex; align-items: center; padding: .12rem .45rem; border-radius: 999px; font-size: .62rem; font-weight: 700; text-transform: uppercase; letter-spacing: .04em; background: #eff6ff; color: #1d4ed8; }}
            .source-chip.shapes {{ background: #fef3c7; color: #b45309; }}
            .source-chip.shacl {{ background: #ede9fe; color: #6d28d9; }}
        </style></head><body><main class="wrap"><section class="hero"><div class="eyebrow">LOD Workbench</div><h1>Validation Report</h1><p class="sub">Conforms: <strong>{conforms}</strong></p></section><section class="grid"><div class="card"><div class="k">Issues</div><div class="v">{issues}</div></div><div class="card ok"><div class="k">Violations</div><div class="v">{violations}</div></div><div class="card warn"><div class="k">Warnings</div><div class="v">{warnings}</div></div><div class="card"><div class="k">Info</div><div class="v">{infos}</div></div></section><section class="panel"><table><thead><tr><th>Severity</th><th>Source</th><th>Line</th><th>Column</th><th>Token</th><th>Message</th><th>Hint</th></tr></thead><tbody>{rows}</tbody></table></section><p class="footer">Generated by LOD Workbench validation. Use the line numbers above to jump back to the RDF source.</p></main></body></html>"#,
        conforms = report.conforms,
        rows = rows,
        issues = issues,
        violations = violations,
        warnings = warnings,
        infos = infos,
    )
}

/// Escape HTML-sensitive characters (&, <, >) to their entity equivalents.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&#38;"),
            '<' => out.push_str("&#60;"),
            '>' => out.push_str("&#62;"),
            _ => out.push(ch),
        }
    }
    out
}
