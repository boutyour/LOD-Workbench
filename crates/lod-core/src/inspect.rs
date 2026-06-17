use crate::{parser, InspectionReport, InspectionRequest, LodError, LodGraph, Node, RdfFormat};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

pub struct InspectionService;

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDF_PROPERTY: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#Property";

impl InspectionService {
    pub fn inspect(&self, req: InspectionRequest) -> Result<InspectionReport, LodError> {
        let fmt = match req.input_format.as_deref() {
            Some(s) => Some(RdfFormat::parse(s)?),
            None => None,
        };
        // The inspection flow always starts from a graph loaded off disk, then
        // reuses the same in-memory summary code as the text-based API.
        let graph = parser::read_graph(&req.input_path, fmt)?;
        self.inspect_graph(graph, req.json_output)
    }

    pub fn inspect_content(
        &self,
        content: &str,
        format: RdfFormat,
        json_output: Option<String>,
    ) -> Result<InspectionReport, LodError> {
        let graph = parser::parse_graph(content, format)?;
        self.inspect_graph(graph, json_output)
    }

    fn inspect_graph(
        &self,
        graph: LodGraph,
        json_output: Option<String>,
    ) -> Result<InspectionReport, LodError> {
        // Aggregate counts and distributions in one pass so the report stays
        // cheap even for larger examples.
        let mut subjects = BTreeSet::new();
        let mut predicates = BTreeSet::new();
        let mut objects = BTreeSet::new();
        let mut iris = 0usize;
        let mut literals = 0usize;
        let mut blank_nodes = 0usize;
        let mut class_distribution = BTreeMap::new();
        let mut property_distribution = BTreeMap::new();

        for t in &graph.triples {
            subjects.insert(format!("{:?}", t.subject));
            predicates.insert(t.predicate.clone());
            objects.insert(format!("{:?}", t.object));
            count_node(&t.subject, &mut iris, &mut literals, &mut blank_nodes);
            count_node(&t.object, &mut iris, &mut literals, &mut blank_nodes);
            *property_distribution.entry(t.predicate.clone()).or_insert(0) += 1;
            if t.predicate == RDF_TYPE {
                if let Node::Iri(class) = &t.object {
                    *class_distribution.entry(class.clone()).or_insert(0) += 1;
                }
            }
        }

        let classes = class_distribution
            .iter()
            .filter(|(k, _)| k.as_str() != RDF_PROPERTY)
            .count();
        let properties = predicates.len();
        let report = InspectionReport {
            triples: graph.triples.len(),
            subjects: subjects.len(),
            predicates: predicates.len(),
            objects: objects.len(),
            iris,
            literals,
            blank_nodes,
            classes,
            properties,
            prefixes: graph.prefixes,
            class_distribution,
            property_distribution,
        };
        if let Some(path) = json_output {
            fs::write(path, serde_json::to_string_pretty(&report)?)?;
        }
        Ok(report)
    }
}

fn count_node(n: &Node, iris: &mut usize, literals: &mut usize, blank_nodes: &mut usize) {
    match n {
        Node::Iri(_) => *iris += 1,
        Node::Literal { .. } => *literals += 1,
        Node::Blank(_) => *blank_nodes += 1,
    }
}
