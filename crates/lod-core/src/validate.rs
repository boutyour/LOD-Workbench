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
    let issues = report.issues.len();
    let violations = report.issues.iter().filter(|i| i.severity.eq_ignore_ascii_case("Violation")).count();
    let warnings = report.issues.iter().filter(|i| i.severity.eq_ignore_ascii_case("Warning")).count();
    let infos = report.issues.iter().filter(|i| i.severity.eq_ignore_ascii_case("Info")).count();
    let rows = if report.issues.is_empty() {
        String::from(r#"<tr><td colspan="3" class="empty">No validation issues were found.</td></tr>"#)
    } else {
        let mut rows = String::new();
        for i in &report.issues {
            rows.push_str(&format!(
                "<tr><td><span class=\"sev sev-{}\">{}</span></td><td class=\"line\">{}</td><td>{}</td></tr>",
                i.severity.to_ascii_lowercase(),
                i.severity,
                i.line.map(|x| x.to_string()).unwrap_or_else(|| "—".into()),
                escape(&i.message)
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
            .line {{ color: var(--muted); width: 90px; }}
            .empty {{ text-align: center; color: var(--muted); padding: 1.5rem 1rem; }}
            .footer {{ margin-top: .85rem; color: var(--muted); font-size: .85rem; }}
        </style></head><body><main class="wrap"><section class="hero"><div class="eyebrow">LOD Workbench</div><h1>Validation Report</h1><p class="sub">Conforms: <strong>{conforms}</strong></p></section><section class="grid"><div class="card"><div class="k">Issues</div><div class="v">{issues}</div></div><div class="card ok"><div class="k">Violations</div><div class="v">{violations}</div></div><div class="card warn"><div class="k">Warnings</div><div class="v">{warnings}</div></div><div class="card"><div class="k">Info</div><div class="v">{infos}</div></div></section><section class="panel"><table><thead><tr><th>Severity</th><th>Line</th><th>Message</th></tr></thead><tbody>{rows}</tbody></table></section><p class="footer">Generated by LOD Workbench validation. Use the line numbers above to jump back to the RDF source.</p></main></body></html>"#,
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
