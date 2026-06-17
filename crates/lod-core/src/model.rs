use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Node {
    Iri(String),
    Literal {
        value: String,
        datatype: Option<String>,
        lang: Option<String>,
    },
    Blank(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Triple {
    pub subject: Node,
    pub predicate: String,
    pub object: Node,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LodGraph {
    #[serde(default)]
    pub base: Option<String>,
    pub prefixes: BTreeMap<String, String>,
    pub triples: Vec<Triple>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "nodeType")]
    pub node_type: String,
    pub color: String,
    pub shape: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationGraph {
    pub nodes: Vec<VisualizationNode>,
    pub edges: Vec<VisualizationEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionReport {
    pub triples: usize,
    pub subjects: usize,
    pub predicates: usize,
    pub objects: usize,
    pub iris: usize,
    pub literals: usize,
    pub blank_nodes: usize,
    pub classes: usize,
    pub properties: usize,
    pub prefixes: BTreeMap<String, String>,
    pub class_distribution: BTreeMap<String, usize>,
    pub property_distribution: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: String,
    pub message: String,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub conforms: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRequest {
    pub input_path: String,
    pub output_path: String,
    pub source_format: Option<String>,
    pub target_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionRequest {
    pub input_path: String,
    pub input_format: Option<String>,
    pub json_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub data_graph_path: String,
    pub shapes_graph_path: Option<String>,
    pub report_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingRequest {
    pub input_path: String,
    pub mapping_path: String,
    pub output_path: String,
    pub output_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationRequest {
    pub input_path: String,
    pub input_format: Option<String>,
    pub output_path: String,
}
