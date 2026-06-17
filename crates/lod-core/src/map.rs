use crate::{parser, LodError, LodGraph, MappingRequest, Node, RdfFormat, Triple};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct MappingConfig {
    pub base_uri: String,
    pub subject_column: String,
    pub class_uri: String,
    #[serde(default)]
    pub prefixes: BTreeMap<String, String>,
    pub columns: BTreeMap<String, ColumnMapping>,
}

#[derive(Debug, Deserialize)]
pub struct ColumnMapping {
    pub predicate: String,
    pub datatype: Option<String>,
    pub lang: Option<String>,
}

pub struct MappingService;

impl MappingService {
    pub fn map_csv_to_rdf(&self, req: MappingRequest) -> Result<(), LodError> {
        let yaml = std::fs::read_to_string(&req.mapping_path)?;
        let cfg: MappingConfig = serde_yaml::from_str(&yaml)?;
        let mut reader = csv::Reader::from_path(&req.input_path)?;
        let headers = reader.headers()?.clone();
        let subject_idx = headers
            .iter()
            .position(|h| h == cfg.subject_column.as_str())
            .ok_or_else(|| LodError::Mapping(format!("subject column `{}` not found", cfg.subject_column)))?;
        let mut graph = LodGraph {
            base: None,
            prefixes: cfg.prefixes.clone(),
            triples: Vec::new(),
        };
        let class_uri = expand(&cfg.class_uri, &cfg.prefixes);
        for result in reader.records() {
            let rec = result?;
            let sid = rec.get(subject_idx).unwrap_or("").trim();
            if sid.is_empty() {
                continue;
            }
            let subject = Node::Iri(format!("{}{}", cfg.base_uri, slug(sid)));
            graph.triples.push(Triple {
                subject: subject.clone(),
                predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
                object: Node::Iri(class_uri.clone()),
            });
            for (col, mapping) in &cfg.columns {
                if let Some(idx) = headers.iter().position(|h| h == col.as_str()) {
                    let value = rec.get(idx).unwrap_or("").trim();
                    if value.is_empty() {
                        continue;
                    }
                    graph.triples.push(Triple {
                        subject: subject.clone(),
                        predicate: expand(&mapping.predicate, &cfg.prefixes),
                        object: Node::Literal {
                            value: value.to_string(),
                            datatype: mapping.datatype.as_ref().map(|d| expand(d, &cfg.prefixes)),
                            lang: mapping.lang.clone(),
                        },
                    });
                }
            }
        }
        let fmt = match req.output_format.as_deref() {
            Some(s) => Some(RdfFormat::parse(s)?),
            None => None,
        };
        parser::write_graph(&graph, &req.output_path, fmt)?;
        Ok(())
    }
}

fn expand(term: &str, prefixes: &BTreeMap<String, String>) -> String {
    if term.starts_with("http://") || term.starts_with("https://") {
        return term.to_string();
    }
    if let Some((p, local)) = term.split_once(':') {
        if let Some(base) = prefixes.get(p) {
            return format!("{base}{local}");
        }
    }
    term.to_string()
}
fn slug(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| if c == ' ' || c == '/' { '-' } else { c })
        .collect()
}
