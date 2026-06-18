use lod_core::{model::*, parser, RdfFormat, ValidationReportFormat, ValidationService};
use std::collections::BTreeMap;
use std::fs;

fn write_temp_file(name: &str, content: &str) -> String {
    let path = std::env::temp_dir().join(format!("{}-{name}", std::process::id()));
    fs::write(&path, content).unwrap();
    path.to_string_lossy().to_string()
}

// ---------------------------------------------------------------------------
// Parser unit tests
// ---------------------------------------------------------------------------

#[test]
fn parses_turtle_subset() {
    let ttl = "@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.triples.len(), 1);
    assert_eq!(graph.prefixes.get("ex").unwrap(), "https://example.org/");
}

#[test]
fn parses_turtle_with_multiple_triples() {
    let ttl = "@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" .\nex:d ex:e \"f\" .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.triples.len(), 2);
}

#[test]
fn parses_turtle_base_and_relative_iris() {
    let ttl = r#"@base <https://example.org/base/> .
<people/ada> <schema/name> "Ada" .
"#;
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.base.as_deref(), Some("https://example.org/base/"));
    assert_eq!(
        graph.triples[0].subject,
        Node::Iri("https://example.org/base/people/ada".into())
    );
}

#[test]
fn parses_turtle_semicolon_chain_and_multiline_statement() {
    let ttl = r#"@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix foaf:   <http://xmlns.com/foaf/0.1/> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://schema.org/> .

@prefix people: <https://example.org/people/> .

people:adaLovelace    rdf:type               foaf:Person ;
  foaf:name "Ada Lovelace" ;
  schema:birthDate "1815-12-10"^^xsd:date ;
  schema:knows people:charlesBabbage ;
  schema:knowsAbout "Programmering"@nb .
"#;
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.triples.len(), 5);
    assert!(graph.triples.iter().any(|t| t.predicate.ends_with("#type")));
    assert!(graph
        .triples
        .iter()
        .any(|t| matches!(&t.object, Node::Literal { lang, .. } if lang.as_deref() == Some("nb"))));
}

#[test]
fn parses_turtle_blank_node_property_lists_and_object_lists() {
    let ttl = r#"@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix foaf:   <http://xmlns.com/foaf/0.1/> .
@prefix schema: <https://schema.org/> .

@prefix people: <https://example.org/people/> .

people:adaLovelace a foaf:Person ;
    schema:children [
      a foaf:Person ;
      foaf:name "Byron" ;
    ] ,
    [
      a foaf:Person ;
      foaf:name "Anna Isabella" ;
    ] ,
    [
      a foaf:Person ;
      foaf:name "Ralph Gordon" ;
    ] .
    "#;
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.triples.len(), 10);
    assert!(graph.triples.iter().any(|t| t.predicate.ends_with("children")));
    assert!(graph.triples.iter().filter(|t| t.predicate.ends_with("name")).count() >= 3);
}

#[test]
fn parses_turtle_rdf_bag_blank_node_property_list() {
    let ttl = r#"@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix s:   <http://example.org/students/vocab#> .

<http://example.org/courses/6.001>
    s:students [
        a rdf:Bag;
        rdf:_1 <http://example.org/students/Amy>;
        rdf:_2 <http://example.org/students/Mohamed>;
        rdf:_3 <http://example.org/students/Johann>;
        rdf:_4 <http://example.org/students/Maria>;
        rdf:_5 <http://example.org/students/Phuong>.
    ].
"#;
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.triples.len(), 7);
    assert!(graph.triples.iter().any(|t| t.predicate.ends_with("students")));
    assert!(graph.triples.iter().any(|t| t.predicate.ends_with("#_5")));
}

#[test]
fn parses_turtle_collections() {
    let ttl = r#"@prefix ex: <https://example.org/> .
ex:a ex:list ( ex:b ex:c "d" ) .
"#;
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert!(graph.triples.iter().any(|t| t.predicate.ends_with("#first")));
    assert!(graph.triples.iter().any(|t| t.predicate.ends_with("#rest")));
    assert!(graph
        .triples
        .iter()
        .any(|t| matches!(&t.object, Node::Literal { value, .. } if value == "d")));
}

#[test]
fn parses_turtle_blank_nodes() {
    let ttl = "@prefix ex: <https://example.org/> .\n_:n1 ex:b _:n2 .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.triples.len(), 1);
    assert!(matches!(&graph.triples[0].subject, Node::Blank(b) if b == "_:n1"));
    assert!(matches!(&graph.triples[0].object, Node::Blank(b) if b == "_:n2"));
}

#[test]
fn parses_turtle_rdf_type_shorthand() {
    let ttl = "@prefix ex: <https://example.org/> .\nex:subject a ex:Class .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(
        graph.triples[0].predicate,
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
    );
}

#[test]
fn parses_turtle_typed_literal() {
    let ttl = "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n<https://example.org/a> <https://example.org/b> \"42\"^^xsd:integer .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    match &graph.triples[0].object {
        Node::Literal { value, datatype, lang } => {
            assert_eq!(value, "42");
            assert_eq!(datatype.as_deref(), Some("http://www.w3.org/2001/XMLSchema#integer"));
            assert!(lang.is_none());
        }
        _ => panic!("expected literal"),
    }
}

#[test]
fn parses_turtle_lang_tagged_literal() {
    let ttl = "@prefix ex: <https://example.org/> .\nex:a ex:b \"hello\"@en .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    match &graph.triples[0].object {
        Node::Literal { value, datatype, lang } => {
            assert_eq!(value, "hello");
            assert!(datatype.is_none());
            assert_eq!(lang.as_deref(), Some("en"));
        }
        _ => panic!("expected literal"),
    }
}

#[test]
fn parses_turtle_inline_comments_in_statements() {
    let ttl = r#"@prefix ex: <https://example.org/> . # prefix comment
ex:a ex:b "c" . # trailing comment
ex:d ex:e [ ex:f "g" ; # inside blank node
  ex:h "i" ] .
"#;
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.triples.len(), 4);
    assert!(graph
        .triples
        .iter()
        .any(|t| matches!(&t.object, Node::Literal { value, .. } if value == "g")));
}

#[test]
fn parses_turtle_escaped_literal() {
    let ttl = "@prefix ex: <https://example.org/> .\nex:a ex:b \"line1\\nline2\" .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    match &graph.triples[0].object {
        Node::Literal { value, .. } => {
            assert_eq!(value, "line1\nline2");
        }
        _ => panic!("expected literal"),
    }
}

#[test]
fn parses_ntriples() {
    let nt = "<https://example.org/a> <https://example.org/b> \"c\" .\n";
    let graph = parser::parse_graph(nt, RdfFormat::NTriples).unwrap();
    assert_eq!(graph.triples.len(), 1);
    assert!(graph.prefixes.is_empty());
}

#[test]
fn parses_ntriples_with_blank_node() {
    let nt = "<https://example.org/a> <https://example.org/b> _:blank1 .\n";
    let graph = parser::parse_graph(nt, RdfFormat::NTriples).unwrap();
    assert!(matches!(&graph.triples[0].object, Node::Blank(b) if b == "_:blank1"));
}

#[test]
fn parses_ntriples_typed_and_language_literals() {
    let nt = r#"<https://example.org/a> <https://example.org/name> "Ada"@en .
<https://example.org/a> <https://example.org/count> "2"^^<http://www.w3.org/2001/XMLSchema#integer> .
"#;
    let graph = parser::parse_graph(nt, RdfFormat::NTriples).unwrap();
    assert_eq!(graph.triples.len(), 2);
    assert!(graph.triples.iter().any(|t| {
        matches!(&t.object, Node::Literal { value, lang: Some(lang), .. } if value == "Ada" && lang == "en")
    }));
    assert!(graph.triples.iter().any(|t| {
        matches!(
            &t.object,
            Node::Literal { value, datatype: Some(datatype), .. }
                if value == "2" && datatype == "http://www.w3.org/2001/XMLSchema#integer"
        )
    }));
}

#[test]
fn parses_jsonld_subset() {
    let jsonld = r#"{"@context":{"ex":"https://example.org/"},"@graph":[{"@id":"ex:a","ex:b":"c"}]}"#;
    let graph = parser::parse_graph(jsonld, RdfFormat::JsonLd).unwrap();
    assert_eq!(graph.triples.len(), 1);
    assert_eq!(graph.prefixes.get("ex").unwrap(), "https://example.org/");
}

#[test]
fn skips_comments_and_empty_lines() {
    let ttl = "# this is a comment\n\n@prefix ex: <https://example.org/> .\n# another comment\nex:a ex:b \"c\" .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.triples.len(), 1);
}

#[test]
fn rejects_invalid_syntax() {
    let ttl = "@prefix ex: <https://example.org/> .\nex:a .\n";
    let result = parser::parse_graph(ttl, RdfFormat::Turtle);
    assert!(result.is_err());
}

#[test]
fn rejects_missing_prefix_terminator_with_clear_message() {
    let ttl = r#"@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#>
"#;
    let err = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap_err().to_string();
    assert!(err.contains("terminator"));
}

#[test]
fn validation_report_includes_location_and_hints() {
    let ttl = r#"@prefix ex: <https://example.org/> .
ex:a ex:b ex:c .
ex:a ex:b ex:c .
"#;
    let svc = ValidationService;
    let report = svc
        .validate_content_with_format(ttl, RdfFormat::Turtle, None, None, Some(ValidationReportFormat::Text))
        .unwrap();
    assert!(report.issues.iter().any(|i| i.suggestion.is_some()));
}

#[test]
fn validation_text_report_is_generated() {
    let ttl = r#"@prefix ex: <https://example.org/> .
ex:a ex:b ex:c .
"#;
    let svc = ValidationService;
    let report = svc
        .validate_content_with_format(ttl, RdfFormat::Turtle, None, None, Some(ValidationReportFormat::Text))
        .unwrap();
    assert!(report.conforms);
}

#[cfg(not(feature = "rudof-shacl"))]
#[test]
fn validation_notes_when_shacl_feature_is_disabled() {
    let ttl = r#"@prefix ex: <https://example.org/> .
ex:a ex:b ex:c .
"#;
    let shapes_path = write_temp_file(
        "lod-workbench-shapes-disabled.ttl",
        r#"@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <https://example.org/> .

ex:AnyShape a sh:NodeShape .
"#,
    );
    let svc = ValidationService;
    let report = svc
        .validate_content(ttl, RdfFormat::Turtle, Some(shapes_path.clone()), None)
        .unwrap();

    assert!(report.conforms);
    assert!(report.issues.iter().any(|i| i.message.contains("rudof-shacl")));
    let _ = fs::remove_file(shapes_path);
}

#[cfg(not(feature = "rudof-shacl"))]
#[test]
fn validation_accepts_inline_shapes_content() {
    let ttl = r#"@prefix ex: <https://example.org/> .
ex:a ex:b ex:c .
"#;
    let shapes = r#"@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <https://example.org/> .

ex:AnyShape a sh:NodeShape .
"#;
    let svc = ValidationService;
    let report = svc
        .validate_content_with_shapes(
            ttl,
            RdfFormat::Turtle,
            Some(shapes),
            Some(RdfFormat::Turtle),
            None,
            Some(ValidationReportFormat::Json),
        )
        .unwrap();

    assert!(report.conforms);
    assert!(report.issues.iter().any(|i| i.message.contains("inline SHACL shapes")));
}

#[cfg(feature = "rudof-shacl")]
#[test]
fn validation_uses_rudof_shacl_when_feature_is_enabled() {
    let ttl = r#"@prefix ex: <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:ada a foaf:Person .
"#;
    let shapes_path = write_temp_file(
        "lod-workbench-shapes-enabled.ttl",
        r#"@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:PersonShape a sh:NodeShape ;
    sh:targetClass foaf:Person ;
    sh:property [
        sh:path foaf:name ;
        sh:minCount 1
    ] .
"#,
    );
    let svc = ValidationService;
    let report = svc
        .validate_content(ttl, RdfFormat::Turtle, Some(shapes_path.clone()), None)
        .unwrap();

    assert!(!report.conforms);
    assert!(report
        .issues
        .iter()
        .any(|i| i.message.contains("SHACL validation failed")));
    let _ = fs::remove_file(shapes_path);
}

#[test]
fn rejects_whitespace_in_object_with_clear_message() {
    let ttl = r#"@prefix : <http://example.org/schemas/vehicles#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:rearSeatLegRoom a rdfs:Class ;
    rdfs:domain rdf:resource :MotorVehicle ;
    rdfs:range "1" .
"#;
    let err = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap_err().to_string();
    assert!(err.contains("unexpected whitespace in object"));
}

#[test]
fn rejects_unknown_format() {
    let result = RdfFormat::parse("unknown-format");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Serialization tests
// ---------------------------------------------------------------------------

#[test]
fn serializes_ntriples() {
    let ttl = "@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    let nt = parser::serialize_graph(&graph, RdfFormat::NTriples).unwrap();
    assert!(nt.contains("<https://example.org/a>"));
    assert!(nt.contains("\"c\""));
}

#[test]
fn serializes_deduplicated_blank_node_graph() {
    let ttl = r#"@prefix ex: <https://example.org/> .
ex:a ex:b [ ex:c "x" ] .
"#;
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    let turtle = parser::serialize_graph(&graph, RdfFormat::Turtle).unwrap();
    let graph2 = parser::parse_graph(&turtle, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.triples.len(), graph2.triples.len());
}

#[test]
fn serializes_turtle() {
    let mut prefixes = BTreeMap::new();
    prefixes.insert("ex".into(), "https://example.org/".into());
    let graph = LodGraph {
        base: None,
        prefixes,
        triples: vec![Triple {
            subject: Node::Iri("https://example.org/a".into()),
            predicate: "https://example.org/b".into(),
            object: Node::Literal {
                value: "test".into(),
                datatype: None,
                lang: None,
            },
        }],
    };
    let turtle = parser::serialize_graph(&graph, RdfFormat::Turtle).unwrap();
    assert!(turtle.contains("@prefix ex: <https://example.org/> ."));
    assert!(turtle.contains("<https://example.org/a>"));
}

#[test]
fn serializes_turtle_with_base() {
    let graph = LodGraph {
        base: Some("https://example.org/base/".into()),
        prefixes: BTreeMap::new(),
        triples: vec![Triple {
            subject: Node::Iri("https://example.org/base/people/ada".into()),
            predicate: "https://example.org/base/schema/name".into(),
            object: Node::Literal {
                value: "Ada".into(),
                datatype: None,
                lang: None,
            },
        }],
    };
    let turtle = parser::serialize_graph(&graph, RdfFormat::Turtle).unwrap();
    assert!(turtle.contains("@base <https://example.org/base/> ."));
}

#[test]
fn serializes_jsonld() {
    let ttl = "@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    let jsonld = parser::serialize_graph(&graph, RdfFormat::JsonLd).unwrap();
    assert!(jsonld.contains("\"@context\""));
    assert!(jsonld.contains("\"@graph\""));
}

#[test]
fn serializes_lang_tagged_literal() {
    let graph = LodGraph {
        base: None,
        prefixes: BTreeMap::new(),
        triples: vec![Triple {
            subject: Node::Iri("https://example.org/a".into()),
            predicate: "https://example.org/name".into(),
            object: Node::Literal {
                value: "hello".into(),
                datatype: None,
                lang: Some("en".into()),
            },
        }],
    };
    let nt = parser::serialize_graph(&graph, RdfFormat::NTriples).unwrap();
    assert!(nt.contains("\"hello\"@en"));
}

#[test]
fn serializes_typed_literal() {
    let graph = LodGraph {
        base: None,
        prefixes: BTreeMap::new(),
        triples: vec![Triple {
            subject: Node::Iri("https://example.org/a".into()),
            predicate: "https://example.org/age".into(),
            object: Node::Literal {
                value: "42".into(),
                datatype: Some("http://www.w3.org/2001/XMLSchema#integer".into()),
                lang: None,
            },
        }],
    };
    let nt = parser::serialize_graph(&graph, RdfFormat::NTriples).unwrap();
    assert!(nt.contains("\"42\"^^<http://www.w3.org/2001/XMLSchema#integer>"));
}

// ---------------------------------------------------------------------------
// Round-trip tests
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_turtle_to_ntriples() {
    let ttl = "@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" .\nex:d ex:e \"f\" .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    let nt = parser::serialize_graph(&graph, RdfFormat::NTriples).unwrap();
    let graph2 = parser::parse_graph(&nt, RdfFormat::NTriples).unwrap();
    assert_eq!(graph.triples.len(), graph2.triples.len());
}

#[test]
fn roundtrip_blank_nodes() {
    let ttl = "@prefix ex: <https://example.org/> .\n_:a ex:b _:c .\n_:d ex:e \"hello\" .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    let nt = parser::serialize_graph(&graph, RdfFormat::NTriples).unwrap();
    let graph2 = parser::parse_graph(&nt, RdfFormat::NTriples).unwrap();
    assert_eq!(graph.triples.len(), graph2.triples.len());
}

// ---------------------------------------------------------------------------
// Format detection tests
// ---------------------------------------------------------------------------

#[test]
fn detects_format_from_extension() {
    assert_eq!(RdfFormat::parse("ttl").unwrap(), RdfFormat::Turtle);
    assert_eq!(RdfFormat::parse("turtle").unwrap(), RdfFormat::Turtle);
    assert_eq!(RdfFormat::parse("nt").unwrap(), RdfFormat::NTriples);
    assert_eq!(RdfFormat::parse("n-triples").unwrap(), RdfFormat::NTriples);
    assert_eq!(RdfFormat::parse("json-ld").unwrap(), RdfFormat::JsonLd);
    assert_eq!(RdfFormat::parse("jsonld").unwrap(), RdfFormat::JsonLd);
}

#[test]
fn detects_format_case_insensitive() {
    assert_eq!(RdfFormat::parse("Turtle").unwrap(), RdfFormat::Turtle);
    assert_eq!(RdfFormat::parse("N-TRIPLES").unwrap(), RdfFormat::NTriples);
    assert_eq!(RdfFormat::parse("JSON-LD").unwrap(), RdfFormat::JsonLd);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_content() {
    let graph = parser::parse_graph("", RdfFormat::Turtle).unwrap();
    assert!(graph.triples.is_empty());
    assert!(graph.prefixes.is_empty());
}

#[test]
fn only_prefixes() {
    let ttl = "@prefix ex: <https://example.org/> .\n@prefix foaf: <http://xmlns.com/foaf/0.1/> .\n";
    let graph = parser::parse_graph(ttl, RdfFormat::Turtle).unwrap();
    assert_eq!(graph.prefixes.len(), 2);
    assert!(graph.triples.is_empty());
}

// ---------------------------------------------------------------------------
// Model equality tests
// ---------------------------------------------------------------------------

#[test]
fn node_equality() {
    let iri1 = Node::Iri("http://example.org/a".into());
    let iri2 = Node::Iri("http://example.org/a".into());
    assert_eq!(iri1, iri2);

    let lit = Node::Literal {
        value: "test".into(),
        datatype: None,
        lang: None,
    };
    assert_ne!(iri1, lit);
}

#[test]
fn triple_ordering() {
    let t1 = Triple {
        subject: Node::Iri("http://example.org/a".into()),
        predicate: "http://example.org/p".into(),
        object: Node::Literal {
            value: "1".into(),
            datatype: None,
            lang: None,
        },
    };
    let t2 = Triple {
        subject: Node::Iri("http://example.org/b".into()),
        predicate: "http://example.org/p".into(),
        object: Node::Literal {
            value: "2".into(),
            datatype: None,
            lang: None,
        },
    };
    assert!(t1 < t2);
}
