use crate::{parser, LodError, LodGraph, Node, RdfFormat, ValidationIssue, ValidationReport, ValidationRequest};
use std::fs;

pub struct ValidationService;

impl ValidationService {
    pub fn validate(&self, req: ValidationRequest) -> Result<ValidationReport, LodError> {
        let content = fs::read_to_string(&req.data_graph_path)?;
        let fmt = RdfFormat::from_path(&req.data_graph_path)?;
        self.validate_content(&content, fmt, req.shapes_graph_path, req.report_path)
    }

    pub fn validate_content(
        &self,
        content: &str,
        fmt: RdfFormat,
        shapes_graph_path: Option<String>,
        report_path: Option<String>,
    ) -> Result<ValidationReport, LodError> {
        let mut issues = Vec::new();
        // V1 validation keeps the logic deliberately lightweight: parse the
        // graph, run IRI checks, and optionally emit a note about SHACL.
        match parser::parse_graph(content, fmt) {
            Ok(graph) => {
                validate_graph(&graph, &mut issues);
                if shapes_graph_path.is_some() {
                    issues.push(ValidationIssue {
                        severity: "Info".into(),
                        message: "SHACL full validation is reserved for V1.1; this V1 performs syntax and IRI checks."
                            .into(),
                        line: None,
                    });
                }
            }
            Err(e) => issues.push(ValidationIssue {
                severity: "Violation".into(),
                message: e.to_string(),
                line: None,
            }),
        }
        let conforms = !issues.iter().any(|i| i.severity == "Violation");
        let report = ValidationReport { conforms, issues };
        if let Some(path) = report_path {
            if path.ends_with(".json") {
                fs::write(path, serde_json::to_string_pretty(&report)?)?;
            } else {
                fs::write(path, validation_html(&report))?;
            }
        }
        Ok(report)
    }
}

fn is_http_iri(iri: &str) -> bool {
    iri.starts_with("http://") || iri.starts_with("https://")
}

fn validate_graph(graph: &LodGraph, issues: &mut Vec<ValidationIssue>) {
    for (idx, t) in graph.triples.iter().enumerate() {
        // Each triple is checked independently so errors can be reported with
        // a simple line number that is useful in the UI.
        if !is_http_iri(&t.predicate) {
            issues.push(ValidationIssue {
                severity: "Warning".into(),
                message: format!("Predicate is not an HTTP IRI: {}", t.predicate),
                line: Some(idx + 1),
            });
        }
        if let Node::Iri(iri) = &t.subject {
            check_iri(iri, idx + 1, issues);
        }
        if let Node::Iri(iri) = &t.object {
            check_iri(iri, idx + 1, issues);
        }
    }
}

fn check_iri(iri: &str, line: usize, issues: &mut Vec<ValidationIssue>) {
    if iri.contains(' ') {
        issues.push(ValidationIssue {
            severity: "Violation".into(),
            message: format!("IRI contains spaces: {iri}"),
            line: Some(line),
        });
    }
    if !is_http_iri(iri) && !iri.starts_with("_:") {
        issues.push(ValidationIssue {
            severity: "Warning".into(),
            message: format!("IRI is not dereferenceable HTTP(S): {iri}"),
            line: Some(line),
        });
    }
}
fn validation_html(report: &ValidationReport) -> String {
    let mut rows = String::new();
    for i in &report.issues {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
            i.severity,
            i.line.map(|x| x.to_string()).unwrap_or_default(),
            escape(&i.message)
        ));
    }
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>LOD Validation Report</title><style>body{{font-family:system-ui;margin:2rem}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #ddd;padding:8px}}th{{background:#f5f5f5}}</style></head><body><h1>LOD Validation Report</h1><p><strong>Conforms:</strong> {conforms}</p><table><thead><tr><th>Severity</th><th>Line</th><th>Message</th></tr></thead><tbody>{rows}</tbody></table></body></html>"#,
        conforms = report.conforms,
        rows = rows,
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
