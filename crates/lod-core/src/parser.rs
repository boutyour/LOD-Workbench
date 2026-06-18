use crate::{LodError, LodGraph, Node, RdfFormat, Triple};
use oxiri::Iri;
use oxrdf::{BlankNodeRef, GraphNameRef, LiteralRef, NamedNodeRef, NamedOrBlankNodeRef, QuadRef, TermRef, TripleRef};
use oxrdfxml::{RdfXmlParser, RdfXmlSerializer};
use oxttl::{TriGParser, TriGSerializer};
use rio_api::model::{Literal, Subject, Term};
use rio_api::parser::TriplesParser;
use rio_turtle::{NTriplesParser, TurtleParser};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Cursor};
use std::path::Path;

pub fn read_graph(path: impl AsRef<Path>, format: Option<RdfFormat>) -> Result<LodGraph, LodError> {
    let path = path.as_ref();
    let fmt = match format {
        Some(f) => f,
        None => RdfFormat::from_path(path)?,
    };
    match fmt {
        RdfFormat::NTriples => {
            let file = fs::File::open(path)?;
            parse_line_based_rdf_reader(BufReader::new(file))
        }
        RdfFormat::RdfXml => {
            let file = fs::File::open(path)?;
            parse_rdfxml_reader(BufReader::new(file))
        }
        RdfFormat::TriG => {
            let file = fs::File::open(path)?;
            parse_trig_reader(BufReader::new(file))
        }
        _ => {
            let content = fs::read_to_string(path)?;
            parse_graph(&content, fmt)
        }
    }
}

pub fn write_graph(graph: &LodGraph, path: impl AsRef<Path>, format: Option<RdfFormat>) -> Result<(), LodError> {
    let path = path.as_ref();
    let fmt = match format {
        Some(f) => f,
        None => RdfFormat::from_path(path)?,
    };
    let content = serialize_graph(graph, fmt)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn parse_graph(content: &str, format: RdfFormat) -> Result<LodGraph, LodError> {
    match format {
        RdfFormat::Turtle => parse_turtle_rdf(content),
        RdfFormat::NTriples => parse_line_based_rdf(content),
        RdfFormat::JsonLd => parse_jsonld_subset(content),
        RdfFormat::RdfXml => parse_rdfxml_content(content),
        RdfFormat::TriG => parse_trig_content(content),
    }
}

pub fn serialize_graph(graph: &LodGraph, format: RdfFormat) -> Result<String, LodError> {
    match format {
        RdfFormat::Turtle => Ok(serialize_turtle(graph)),
        RdfFormat::NTriples => Ok(serialize_ntriples(graph)),
        RdfFormat::JsonLd => serialize_jsonld(graph),
        RdfFormat::RdfXml => serialize_rdfxml(graph),
        RdfFormat::TriG => serialize_trig(graph),
    }
}

fn parse_line_based_rdf(content: &str) -> Result<LodGraph, LodError> {
    parse_line_based_rdf_reader(BufReader::new(Cursor::new(content.as_bytes())))
}

fn parse_line_based_rdf_reader<R: BufRead>(reader: R) -> Result<LodGraph, LodError> {
    let mut graph = LodGraph::default();
    NTriplesParser::new(reader)
        .parse_all(&mut |triple| {
            graph.triples.push(Triple {
                subject: rio_subject_to_node(&triple.subject),
                predicate: triple.predicate.iri.to_string(),
                object: rio_term_to_node(&triple.object),
            });
            Ok(()) as Result<(), rio_turtle::TurtleError>
        })
        .map_err(|e| LodError::RdfParsing(e.to_string()))?;
    Ok(graph)
}

fn parse_rdfxml_reader<R: BufRead>(reader: R) -> Result<LodGraph, LodError> {
    let mut graph = LodGraph::default();
    for triple in RdfXmlParser::new().for_reader(reader) {
        let triple = triple.map_err(|e| LodError::RdfParsing(e.to_string()))?;
        graph.triples.push(Triple {
            subject: oxrdf_named_or_blank_to_node(triple.subject.as_ref()),
            predicate: triple.predicate.as_str().to_string(),
            object: oxrdf_term_to_node(triple.object.as_ref()),
        });
    }
    Ok(graph)
}

fn parse_trig_reader<R: BufRead>(reader: R) -> Result<LodGraph, LodError> {
    let mut graph = LodGraph::default();
    let mut parser = TriGParser::new().for_reader(reader);
    for quad in parser.by_ref() {
        let quad = quad.map_err(|e| LodError::RdfParsing(e.to_string()))?;
        push_quad(&mut graph, quad);
    }
    if let Some(base) = parser.base_iri() {
        graph.base = Some(base.to_string());
    }
    for (prefix, iri) in parser.prefixes() {
        graph.prefixes.insert(prefix.to_string(), iri.to_string());
    }
    Ok(graph)
}

fn parse_rdfxml_content(content: &str) -> Result<LodGraph, LodError> {
    let mut graph = LodGraph::default();
    for triple in RdfXmlParser::new().for_slice(content.as_bytes()) {
        let triple = triple.map_err(|e| LodError::RdfParsing(e.to_string()))?;
        graph.triples.push(Triple {
            subject: oxrdf_named_or_blank_to_node(triple.subject.as_ref()),
            predicate: triple.predicate.as_str().to_string(),
            object: oxrdf_term_to_node(triple.object.as_ref()),
        });
    }
    Ok(graph)
}

fn parse_trig_content(content: &str) -> Result<LodGraph, LodError> {
    let mut graph = LodGraph::default();
    let mut parser = TriGParser::new().for_slice(content.as_bytes());
    for quad in parser.by_ref() {
        let quad = quad.map_err(|e| LodError::RdfParsing(e.to_string()))?;
        push_quad(&mut graph, quad);
    }
    Ok(graph)
}

fn parse_turtle_rdf(content: &str) -> Result<LodGraph, LodError> {
    match parse_turtle_with_rio(content) {
        Ok(graph) => Ok(graph),
        Err(rio_error) => {
            // The lightweight parser below preserves older, project-specific
            // diagnostics for a few malformed examples while rio_turtle gives
            // us standards coverage for valid Turtle.
            match parse_turtle_subset(content) {
                Ok(graph) => Ok(graph),
                Err(subset_error) => {
                    if subset_error.to_string().contains("line ") {
                        Err(subset_error)
                    } else {
                        Err(rio_error)
                    }
                }
            }
        }
    }
}

fn parse_turtle_with_rio(content: &str) -> Result<LodGraph, LodError> {
    let mut graph = LodGraph::default();
    scan_turtle_declarations(content, &mut graph);
    let base_iri = graph.base.as_ref().and_then(|base| Iri::parse(base.clone()).ok());
    TurtleParser::new(Cursor::new(content.as_bytes()), base_iri)
        .parse_all(&mut |triple| {
            graph.triples.push(Triple {
                subject: rio_subject_to_node(&triple.subject),
                predicate: triple.predicate.iri.to_string(),
                object: rio_term_to_node(&triple.object),
            });
            Ok(()) as Result<(), rio_turtle::TurtleError>
        })
        .map_err(|e| LodError::RdfParsing(e.to_string()))?;
    Ok(graph)
}

fn scan_turtle_declarations(content: &str, graph: &mut LodGraph) {
    for (_, stmt) in split_turtle_statements(content) {
        let trimmed = strip_inline_comment(&stmt);
        let trimmed = trimmed.trim();
        if trimmed.starts_with("@prefix") || trimmed.to_ascii_uppercase().starts_with("PREFIX") {
            let _ = parse_prefix(trimmed, &mut graph.prefixes);
        } else if trimmed.starts_with("@base") || trimmed.to_ascii_uppercase().starts_with("BASE") {
            let _ = parse_base(trimmed, &mut graph.base);
        }
    }
}

fn rio_subject_to_node(subject: &Subject<'_>) -> Node {
    match subject {
        Subject::NamedNode(node) => Node::Iri(node.iri.to_string()),
        Subject::BlankNode(node) => Node::Blank(format!("_:{}", node.id)),
        Subject::Triple(_) => Node::Blank("_:rdfstar_subject".to_string()),
    }
}

fn rio_term_to_node(term: &Term<'_>) -> Node {
    match term {
        Term::NamedNode(node) => Node::Iri(node.iri.to_string()),
        Term::BlankNode(node) => Node::Blank(format!("_:{}", node.id)),
        Term::Literal(literal) => rio_literal_to_node(literal),
        Term::Triple(_) => Node::Blank("_:rdfstar_object".to_string()),
    }
}

fn oxrdf_named_or_blank_to_node(node: oxrdf::NamedOrBlankNodeRef<'_>) -> Node {
    match node {
        oxrdf::NamedOrBlankNodeRef::NamedNode(node) => Node::Iri(node.as_str().to_string()),
        oxrdf::NamedOrBlankNodeRef::BlankNode(node) => Node::Blank(format!("_:{}", node.as_str())),
    }
}

fn oxrdf_term_to_node(term: oxrdf::TermRef<'_>) -> Node {
    match term {
        oxrdf::TermRef::NamedNode(node) => Node::Iri(node.as_str().to_string()),
        oxrdf::TermRef::BlankNode(node) => Node::Blank(format!("_:{}", node.as_str())),
        oxrdf::TermRef::Literal(literal) => Node::Literal {
            value: literal.value().to_string(),
            datatype: Some(literal.datatype().as_str().to_string()),
            lang: literal.language().map(|lang| lang.to_string()),
        },
        _ => Node::Blank("_:rdfstar_object".to_string()),
    }
}

fn push_quad(graph: &mut LodGraph, quad: oxrdf::Quad) {
    let triple = Triple {
        subject: oxrdf_named_or_blank_to_node(quad.subject.as_ref()),
        predicate: quad.predicate.as_str().to_string(),
        object: oxrdf_term_to_node(quad.object.as_ref()),
    };
    match quad.graph_name {
        oxrdf::GraphName::DefaultGraph => graph.triples.push(triple),
        oxrdf::GraphName::NamedNode(node) => {
            graph
                .named_graphs
                .entry(node.as_str().to_string())
                .or_default()
                .push(triple);
        }
        oxrdf::GraphName::BlankNode(node) => {
            graph
                .named_graphs
                .entry(format!("_:{}", node.as_str()))
                .or_default()
                .push(triple);
        }
    }
}

fn rio_literal_to_node(literal: &Literal<'_>) -> Node {
    match literal {
        Literal::Simple { value } => Node::Literal {
            value: value.to_string(),
            datatype: None,
            lang: None,
        },
        Literal::LanguageTaggedString { value, language } => Node::Literal {
            value: value.to_string(),
            datatype: None,
            lang: Some(language.to_string()),
        },
        Literal::Typed { value, datatype } => Node::Literal {
            value: value.to_string(),
            datatype: Some(datatype.iri.to_string()),
            lang: None,
        },
    }
}

fn parse_turtle_subset(content: &str) -> Result<LodGraph, LodError> {
    let mut graph = LodGraph::default();
    let mut blank_id_seq = 0usize;
    for (line_no, stmt) in split_turtle_statements(content) {
        let trimmed = strip_inline_comment(&stmt);
        let trimmed = trimmed.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("@prefix") || trimmed.to_ascii_uppercase().starts_with("PREFIX") {
            parse_prefix(trimmed, &mut graph.prefixes)
                .map_err(|e| LodError::RdfParsing(format!("line {line_no}: {e}")))?;
            continue;
        }
        if trimmed.starts_with("@base") || trimmed.to_ascii_uppercase().starts_with("BASE") {
            parse_base(trimmed, &mut graph.base).map_err(|e| LodError::RdfParsing(format!("line {line_no}: {e}")))?;
            continue;
        }
        let triples = parse_turtle_statement(trimmed, &graph.prefixes, graph.base.as_deref(), &mut blank_id_seq)
            .map_err(|e| LodError::RdfParsing(format!("line {line_no}: {e}")))?;
        graph.triples.extend(triples);
    }
    Ok(graph)
}

fn parse_prefix(line: &str, prefixes: &mut BTreeMap<String, String>) -> Result<(), String> {
    let s = strip_inline_comment(line);
    let cleaned = s.trim().trim_end_matches('.').trim();
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    if parts.len() < 3 {
        return Err("invalid prefix declaration".into());
    }
    if parts.len() > 3 {
        return Err("prefix declaration appears to be missing a `.` terminator".into());
    }
    if !parts[1].ends_with(':') {
        return Err("prefix declaration must use the form `@prefix name: <iri> .`".into());
    }
    let prefix = parts[1].trim_end_matches(':').to_string();
    let iri = trim_iri(parts[2]).ok_or("invalid prefix IRI")?.to_string();
    prefixes.insert(prefix, iri);
    Ok(())
}

fn parse_base(line: &str, base: &mut Option<String>) -> Result<(), String> {
    let s = strip_inline_comment(line);
    let cleaned = s.trim().trim_end_matches('.').trim();
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("invalid base declaration".into());
    }
    if parts.len() > 2 {
        return Err("base declaration appears to be missing a `.` terminator".into());
    }
    if !parts[0].eq_ignore_ascii_case("@base") && !parts[0].eq_ignore_ascii_case("base") {
        return Err("base declaration must start with `@base` or `BASE`".into());
    }
    let iri = trim_iri(parts[1]).ok_or("invalid base IRI")?.to_string();
    *base = Some(iri);
    Ok(())
}

fn parse_turtle_statement(
    stmt: &str,
    prefixes: &BTreeMap<String, String>,
    base: Option<&str>,
    blank_id_seq: &mut usize,
) -> Result<Vec<Triple>, String> {
    let clauses = split_top_level(stmt, ';');
    let mut clauses = clauses.into_iter().filter(|c| !c.trim().is_empty());
    let first = clauses.next().ok_or("missing subject")?;
    let (subject_token, rest) = take_token(first.trim()).ok_or("missing subject")?;
    let subject = parse_subject(subject_token, prefixes, base)?;
    let mut triples = Vec::new();
    parse_predicate_object_list(subject.clone(), rest.trim(), prefixes, base, blank_id_seq, &mut triples)?;

    for clause in clauses {
        parse_predicate_object_list(
            subject.clone(),
            clause.trim(),
            prefixes,
            base,
            blank_id_seq,
            &mut triples,
        )?;
    }

    if triples.is_empty() {
        return Err("missing predicate".into());
    }
    Ok(triples)
}

fn parse_predicate_object_list(
    subject: Node,
    text: &str,
    prefixes: &BTreeMap<String, String>,
    base: Option<&str>,
    blank_id_seq: &mut usize,
    triples: &mut Vec<Triple>,
) -> Result<(), String> {
    let text = strip_inline_comment(text);
    let text = trim_trailing_terminators(&text);
    if text.is_empty() {
        return Err("missing predicate".into());
    }
    let (predicate_token, rest) = take_token(text).ok_or("missing predicate")?;
    let predicate = expand_term(predicate_token, prefixes, base)?;
    let objects = split_top_level(rest.trim(), ',');
    if objects.is_empty() {
        return Err("missing object".into());
    }
    for obj in objects {
        let obj = trim_trailing_terminators(&obj);
        if obj.is_empty() {
            return Err("missing object".into());
        }
        let object = parse_object(obj, prefixes, base, blank_id_seq, triples)?;
        triples.push(Triple {
            subject: subject.clone(),
            predicate: predicate.clone(),
            object,
        });
    }
    Ok(())
}

fn take_token(input: &str) -> Option<(&str, &str)> {
    let input = input.trim_start();
    if input.is_empty() {
        return None;
    }

    let mut in_string = false;
    let mut in_iri = false;
    let mut escaped = false;
    let mut depth = 0isize;

    for (idx, ch) in input.char_indices() {
        match ch {
            '"' if !in_iri && !escaped => {
                in_string = !in_string;
            }
            '<' if !in_string => {
                in_iri = true;
            }
            '>' if in_iri && !escaped => {
                in_iri = false;
            }
            '[' | '(' if !in_string && !in_iri => {
                depth += 1;
            }
            ']' | ')' if !in_string && !in_iri => {
                if depth == 0 {
                    return Some((&input[..idx], &input[idx..]));
                }
                depth -= 1;
                if depth == 0 {
                    let end = idx + ch.len_utf8();
                    return Some((&input[..end], &input[end..]));
                }
            }
            c if !in_string && !in_iri && depth == 0 && (c.is_whitespace() || matches!(c, ';' | ',' | '.')) => {
                return Some((&input[..idx], &input[idx..]));
            }
            _ => {}
        }
        if ch != '\\' {
            escaped = false;
        } else if in_string && !escaped {
            escaped = true;
        }
    }

    Some((input, ""))
}

fn split_turtle_statements(content: &str) -> Vec<(usize, String)> {
    // Split on top-level dots while preserving dots inside IRIs, literals,
    // blank node blocks, and RDF collections.
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut line = 1usize;
    let mut start_line = 1usize;
    let mut started = false;
    let mut in_string = false;
    let mut in_iri = false;
    let mut escaped = false;
    let mut in_comment = false;
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

    for ch in content.chars() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
                line += 1;
                if started {
                    buf.push(ch);
                }
            }
            continue;
        }

        if ch == '\n' {
            line += 1;
        }

        if !started && !ch.is_whitespace() && ch != '#' {
            started = true;
            start_line = line;
        }

        if !in_string && !in_iri && ch == '#' {
            in_comment = true;
            continue;
        }

        match ch {
            '"' if !in_iri && !escaped => {
                in_string = !in_string;
                buf.push(ch);
                escaped = false;
            }
            '<' if !in_string => {
                in_iri = true;
                buf.push(ch);
                escaped = false;
            }
            '>' if in_iri && !escaped => {
                in_iri = false;
                buf.push(ch);
                escaped = false;
            }
            '[' if !in_string && !in_iri => {
                bracket_depth += 1;
                buf.push(ch);
                escaped = false;
            }
            ']' if !in_string && !in_iri => {
                bracket_depth = bracket_depth.saturating_sub(1);
                buf.push(ch);
                escaped = false;
            }
            '(' if !in_string && !in_iri => {
                paren_depth += 1;
                buf.push(ch);
                escaped = false;
            }
            ')' if !in_string && !in_iri => {
                paren_depth = paren_depth.saturating_sub(1);
                buf.push(ch);
                escaped = false;
            }
            '.' if !in_string && !in_iri => {
                if bracket_depth == 0 && paren_depth == 0 && !buf.trim().is_empty() {
                    out.push((start_line, buf.trim().to_string()));
                    buf.clear();
                    started = false;
                    escaped = false;
                } else {
                    buf.push(ch);
                    escaped = false;
                }
            }
            '\\' if in_string && !escaped => {
                buf.push(ch);
                escaped = true;
                continue;
            }
            _ => {
                buf.push(ch);
                escaped = false;
            }
        }
    }

    if !buf.trim().is_empty() {
        out.push((start_line, buf.trim().to_string()));
    }
    out
}

fn split_top_level(input: &str, delim: char) -> Vec<String> {
    // Split a clause only when we are not inside strings, IRIs, or nested RDF
    // structures such as blank nodes and collections.
    let mut parts = Vec::new();
    let mut buf = String::new();
    let mut in_string = false;
    let mut in_iri = false;
    let mut escaped = false;
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

    for ch in input.chars() {
        match ch {
            '"' if !in_iri && !escaped => {
                in_string = !in_string;
                buf.push(ch);
                escaped = false;
            }
            '<' if !in_string => {
                in_iri = true;
                buf.push(ch);
                escaped = false;
            }
            '>' if in_iri && !escaped => {
                in_iri = false;
                buf.push(ch);
                escaped = false;
            }
            '[' if !in_string && !in_iri => {
                bracket_depth += 1;
                buf.push(ch);
                escaped = false;
            }
            ']' if !in_string && !in_iri => {
                bracket_depth = bracket_depth.saturating_sub(1);
                buf.push(ch);
                escaped = false;
            }
            '(' if !in_string && !in_iri => {
                paren_depth += 1;
                buf.push(ch);
                escaped = false;
            }
            ')' if !in_string && !in_iri => {
                paren_depth = paren_depth.saturating_sub(1);
                buf.push(ch);
                escaped = false;
            }
            '\\' if in_string && !escaped => {
                buf.push(ch);
                escaped = true;
                continue;
            }
            c if c == delim && !in_string && !in_iri && bracket_depth == 0 && paren_depth == 0 => {
                parts.push(buf.trim().to_string());
                buf.clear();
                escaped = false;
                continue;
            }
            _ => {
                buf.push(ch);
                escaped = false;
            }
        }
    }
    if !buf.trim().is_empty() {
        parts.push(buf.trim().to_string());
    }
    parts
}

fn parse_object(
    o: &str,
    prefixes: &BTreeMap<String, String>,
    base: Option<&str>,
    blank_id_seq: &mut usize,
    triples: &mut Vec<Triple>,
) -> Result<Node, String> {
    let o = strip_inline_comment(o);
    let o = trim_trailing_terminators(&o);
    // Choose the parser based on the syntactic form of the object token.
    if contains_bare_whitespace(o)
        && !o.starts_with('<')
        && !o.starts_with('"')
        && !o.starts_with('[')
        && !o.starts_with('(')
        && !o.starts_with("_:")
    {
        return Err("unexpected whitespace in object".into());
    }
    if o.starts_with('[') {
        parse_blank_node_property_list(o, prefixes, base, blank_id_seq, triples)
    } else if o.starts_with('(') {
        parse_rdf_collection(o, prefixes, base, blank_id_seq, triples)
    } else if o.starts_with('"') {
        parse_literal(o, prefixes, base)
    } else if o.starts_with("_:") {
        Ok(Node::Blank(o.to_string()))
    } else {
        Ok(Node::Iri(expand_term(o, prefixes, base)?))
    }
}

fn parse_blank_node_property_list(
    input: &str,
    prefixes: &BTreeMap<String, String>,
    base: Option<&str>,
    blank_id_seq: &mut usize,
    triples: &mut Vec<Triple>,
) -> Result<Node, String> {
    let (inner, rest) = extract_bracket_content(input)?;
    if !rest.trim().is_empty() {
        return Err("unexpected trailing content after blank node property list".into());
    }
    // Emit one fresh blank node and attach each predicate/object clause to it.
    let subject = Node::Blank(fresh_blank_id(blank_id_seq));
    let clauses = split_top_level(inner.trim(), ';');
    let mut saw_predicate = false;
    for clause in clauses {
        let clause = strip_inline_comment(&clause);
        let clause = clause.trim();
        if clause.is_empty() {
            continue;
        }
        saw_predicate = true;
        parse_predicate_object_list(subject.clone(), clause, prefixes, base, blank_id_seq, triples)?;
    }
    if !saw_predicate {
        return Err("empty blank node property list".into());
    }
    Ok(subject)
}

fn parse_rdf_collection(
    input: &str,
    prefixes: &BTreeMap<String, String>,
    base: Option<&str>,
    blank_id_seq: &mut usize,
    triples: &mut Vec<Triple>,
) -> Result<Node, String> {
    let (inner, rest) = extract_group_content(input, '(', ')')?;
    if !rest.trim().is_empty() {
        return Err("unexpected trailing content after RDF collection".into());
    }
    // Build the RDF list cell-by-cell so the serializer can round-trip it.
    let mut items = Vec::new();
    let mut cursor = inner.trim();
    while !cursor.trim().is_empty() {
        let (frag, next) = take_object_fragment(cursor).ok_or("malformed RDF collection")?;
        let frag = strip_inline_comment(frag);
        items.push(frag.trim().to_string());
        cursor = next.trim_start();
    }
    if items.is_empty() {
        return Ok(Node::Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil".to_string()));
    }

    let head = Node::Blank(fresh_blank_id(blank_id_seq));
    let mut current = head.clone();
    for (index, item) in items.iter().enumerate() {
        let next_node = if index + 1 == items.len() {
            Node::Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil".to_string())
        } else {
            Node::Blank(fresh_blank_id(blank_id_seq))
        };
        let value = parse_object(item, prefixes, base, blank_id_seq, triples)?;
        triples.push(Triple {
            subject: current.clone(),
            predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#first".to_string(),
            object: value,
        });
        triples.push(Triple {
            subject: current.clone(),
            predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest".to_string(),
            object: next_node.clone(),
        });
        current = next_node;
    }
    Ok(head)
}

fn extract_bracket_content(input: &str) -> Result<(&str, &str), String> {
    extract_group_content(input, '[', ']')
}

fn extract_group_content(input: &str, open: char, close: char) -> Result<(&str, &str), String> {
    let trimmed = input.trim();
    if !trimmed.starts_with(open) {
        return Err("expected grouped content".into());
    }
    let chars: Vec<char> = trimmed.chars().collect();
    let mut in_string = false;
    let mut in_iri = false;
    let mut escaped = false;
    let mut depth = 0isize;
    let mut start = None;

    for (i, &ch) in chars.iter().enumerate() {
        match ch {
            '"' if !in_iri && !escaped => {
                in_string = !in_string;
                escaped = false;
            }
            '<' if !in_string => {
                in_iri = true;
                escaped = false;
            }
            '>' if in_iri && !escaped => {
                in_iri = false;
                escaped = false;
            }
            '\\' if in_string && !escaped => {
                escaped = true;
                continue;
            }
            c if c == open && !in_string && !in_iri => {
                depth += 1;
                if depth == 1 {
                    start = Some(i + 1);
                }
            }
            c if c == close && !in_string && !in_iri => {
                depth -= 1;
                if depth == 0 {
                    let s = start.ok_or("malformed blank node property list")?;
                    return Ok((&trimmed[s..i], &trimmed[i + 1..]));
                }
            }
            _ => {}
        }
        if ch != '\\' {
            escaped = false;
        }
    }
    Err("unterminated blank node property list".into())
}

fn fresh_blank_id(blank_id_seq: &mut usize) -> String {
    let id = format!("_:b{}", *blank_id_seq);
    *blank_id_seq += 1;
    id
}

fn trim_trailing_terminators(input: &str) -> &str {
    let mut s = input.trim();
    loop {
        let trimmed = s.trim_end();
        let Some(last) = trimmed.chars().last() else {
            return trimmed;
        };
        if matches!(last, '.' | ';' | ',') {
            s = &trimmed[..trimmed.len() - last.len_utf8()];
            continue;
        }
        return trimmed;
    }
}

fn strip_inline_comment(input: &str) -> String {
    let mut out = String::new();
    let mut in_string = false;
    let mut in_iri = false;
    let mut escaped = false;
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

    for ch in input.chars() {
        if ch == '#' && !in_string && !in_iri && bracket_depth == 0 && paren_depth == 0 {
            break;
        }
        match ch {
            '"' if !in_iri && !escaped => in_string = !in_string,
            '<' if !in_string => in_iri = true,
            '>' if in_iri && !escaped => in_iri = false,
            '[' if !in_string && !in_iri => bracket_depth += 1,
            ']' if !in_string && !in_iri => bracket_depth = bracket_depth.saturating_sub(1),
            '(' if !in_string && !in_iri => paren_depth += 1,
            ')' if !in_string && !in_iri => paren_depth = paren_depth.saturating_sub(1),
            '\\' if in_string && !escaped => {
                escaped = true;
                out.push(ch);
                continue;
            }
            _ => {}
        }
        out.push(ch);
        if ch != '\\' {
            escaped = false;
        }
    }
    out
}

fn contains_bare_whitespace(input: &str) -> bool {
    input.chars().any(|c| c.is_whitespace())
}

fn take_object_fragment(input: &str) -> Option<(&str, &str)> {
    let input = input.trim_start();
    if input.is_empty() {
        return None;
    }
    let mut in_string = false;
    let mut in_iri = false;
    let mut escaped = false;
    let mut depth = 0isize;
    let mut start = 0usize;

    for (i, ch) in input.char_indices() {
        match ch {
            '"' if !in_iri && !escaped => {
                in_string = !in_string;
            }
            '<' if !in_string => {
                in_iri = true;
            }
            '>' if in_iri && !escaped => {
                in_iri = false;
            }
            '[' | '(' if !in_string && !in_iri => {
                depth += 1;
            }
            ']' | ')' if !in_string && !in_iri => {
                if depth == 0 {
                    return Some((&input[start..i], &input[i..]));
                }
                depth -= 1;
                if depth == 0 {
                    let end = i + ch.len_utf8();
                    return Some((&input[start..end], &input[end..]));
                }
            }
            c if !in_string && !in_iri && depth == 0 && (c.is_whitespace() || matches!(c, ';' | ',' | '.')) => {
                if i == start {
                    start = i + ch.len_utf8();
                    continue;
                }
                let end = i;
                return Some((&input[start..end], &input[end..]));
            }
            _ => {}
        }
        if ch != '\\' {
            escaped = false;
        } else if in_string && !escaped {
            escaped = true;
        }
    }
    if start < input.len() {
        Some((&input[start..], ""))
    } else {
        None
    }
}

fn parse_subject(s: &str, prefixes: &BTreeMap<String, String>, base: Option<&str>) -> Result<Node, String> {
    if s.starts_with("_:") {
        Ok(Node::Blank(s.to_string()))
    } else {
        Ok(Node::Iri(expand_term(s, prefixes, base)?))
    }
}

fn parse_literal(o: &str, prefixes: &BTreeMap<String, String>, base: Option<&str>) -> Result<Node, String> {
    let mut escaped = false;
    let chars: Vec<char> = o.chars().collect();
    let mut end = None;
    for (i, &c) in chars.iter().enumerate().skip(1) {
        if c == '\\' && !escaped {
            escaped = true;
            continue;
        }
        if c == '"' && !escaped {
            end = Some(i);
            break;
        }
        escaped = false;
    }
    let end = end.ok_or("unterminated literal")?;
    let value: String = chars[1..end].iter().collect();
    let suffix: String = chars[end + 1..].iter().collect();
    let suffix = suffix.trim();
    if let Some(lang) = suffix.strip_prefix('@') {
        Ok(Node::Literal {
            value,
            datatype: None,
            lang: Some(lang.to_string()),
        })
    } else if let Some(dt) = suffix.strip_prefix("^^") {
        Ok(Node::Literal {
            value,
            datatype: Some(expand_term(dt.trim(), prefixes, base)?),
            lang: None,
        })
    } else {
        Ok(Node::Literal {
            value,
            datatype: None,
            lang: None,
        })
    }
}

fn expand_term(term: &str, prefixes: &BTreeMap<String, String>, base: Option<&str>) -> Result<String, String> {
    if let Some(iri) = trim_iri(term) {
        return Ok(resolve_iri(iri, base));
    }
    if term == "a" {
        return Ok("http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string());
    }
    if let Some((prefix, local)) = term.split_once(':') {
        if let Some(base) = prefixes.get(prefix) {
            return Ok(format!("{base}{local}"));
        }
        return Err(format!("unknown prefix `{prefix}`"));
    }
    if term.starts_with("http://") || term.starts_with("https://") {
        return Ok(term.to_string());
    }
    if let Some(base_iri) = base {
        if !term.is_empty() {
            return Ok(resolve_relative(base_iri, term));
        }
    }
    Err(format!("cannot expand term `{term}`"))
}

fn trim_iri(token: &str) -> Option<&str> {
    token.strip_prefix('<')?.strip_suffix('>')
}

fn resolve_iri(iri: &str, base: Option<&str>) -> String {
    if iri.starts_with("http://")
        || iri.starts_with("https://")
        || iri.starts_with("urn:")
        || iri.starts_with("mailto:")
    {
        return iri.to_string();
    }
    if let Some(base) = base {
        return resolve_relative(base, iri);
    }
    iri.to_string()
}

fn resolve_relative(base: &str, rel: &str) -> String {
    if rel.starts_with("http://") || rel.starts_with("https://") {
        return rel.to_string();
    }
    if rel.starts_with('#') || rel.starts_with('?') {
        return format!("{base}{rel}");
    }
    if base.ends_with('/') || base.ends_with('#') {
        return format!("{base}{rel}");
    }
    format!("{base}/{rel}")
}

fn serialize_turtle(graph: &LodGraph) -> String {
    let mut out = String::new();
    if let Some(base) = &graph.base {
        out.push_str(&format!("@base <{base}> .\n"));
    }
    for (p, iri) in &graph.prefixes {
        out.push_str(&format!("@prefix {p}: <{iri}> .\n"));
    }
    if !graph.prefixes.is_empty() || graph.base.is_some() {
        out.push('\n');
    }
    for t in normalized_triples(graph) {
        out.push_str(&format!(
            "{} <{}> {} .\n",
            node_to_turtle(&t.subject),
            t.predicate,
            node_to_turtle(&t.object)
        ));
    }
    out
}

fn serialize_ntriples(graph: &LodGraph) -> String {
    normalized_triples(graph)
        .iter()
        .map(|t| {
            format!(
                "{} <{}> {} .",
                node_to_nt(&t.subject),
                t.predicate,
                node_to_nt(&t.object)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn serialize_rdfxml(graph: &LodGraph) -> Result<String, LodError> {
    if !graph.named_graphs.is_empty() {
        return Err(LodError::Validation(
            "RDF/XML does not support named graphs; convert the dataset to TriG first".into(),
        ));
    }
    let mut serializer = RdfXmlSerializer::new();
    if let Some(base) = &graph.base {
        serializer = serializer
            .with_base_iri(base.clone())
            .map_err(|e| LodError::RdfParsing(e.to_string()))?;
    }
    for (prefix, iri) in &graph.prefixes {
        serializer = serializer
            .with_prefix(prefix.clone(), iri.clone())
            .map_err(|e| LodError::RdfParsing(e.to_string()))?;
    }
    let mut serializer = serializer.for_writer(Vec::new());
    for triple in normalized_triples(graph) {
        serializer
            .serialize_triple(triple_to_oxrdf_triple_ref(&triple)?)
            .map_err(|e| LodError::RdfParsing(e.to_string()))?;
    }
    let bytes = serializer.finish().map_err(|e| LodError::RdfParsing(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| LodError::RdfParsing(e.to_string()))
}

fn serialize_trig(graph: &LodGraph) -> Result<String, LodError> {
    let mut serializer = TriGSerializer::new();
    if let Some(base) = &graph.base {
        serializer = serializer
            .with_base_iri(base.clone())
            .map_err(|e| LodError::RdfParsing(e.to_string()))?;
    }
    for (prefix, iri) in &graph.prefixes {
        serializer = serializer
            .with_prefix(prefix.clone(), iri.clone())
            .map_err(|e| LodError::RdfParsing(e.to_string()))?;
    }
    let mut serializer = serializer.for_writer(Vec::new());
    for triple in normalized_triples(&LodGraph {
        base: graph.base.clone(),
        prefixes: graph.prefixes.clone(),
        triples: graph.triples.clone(),
        named_graphs: BTreeMap::new(),
    }) {
        serializer
            .serialize_quad(triple_to_oxrdf_quad_ref(&triple, GraphNameRef::DefaultGraph)?)
            .map_err(|e| LodError::RdfParsing(e.to_string()))?;
    }
    for (graph_name, triples) in &graph.named_graphs {
        let graph_name = graph_name_to_ref(graph_name)?;
        for triple in normalized_triples(&LodGraph {
            base: None,
            prefixes: BTreeMap::new(),
            triples: triples.clone(),
            named_graphs: BTreeMap::new(),
        }) {
            serializer
                .serialize_quad(triple_to_oxrdf_quad_ref(&triple, graph_name)?)
                .map_err(|e| LodError::RdfParsing(e.to_string()))?;
        }
    }
    let bytes = serializer.finish().map_err(|e| LodError::RdfParsing(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| LodError::RdfParsing(e.to_string()))
}

fn node_to_turtle(n: &Node) -> String {
    node_to_nt(n)
}
fn node_to_nt(n: &Node) -> String {
    match n {
        Node::Iri(i) => format!("<{i}>"),
        Node::Blank(b) => b.clone(),
        Node::Literal { value, datatype, lang } => {
            let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
            if let Some(l) = lang {
                format!("\"{escaped}\"@{l}")
            } else if let Some(dt) = datatype {
                format!("\"{escaped}\"^^<{dt}>")
            } else {
                format!("\"{escaped}\"")
            }
        }
    }
}

fn serialize_jsonld(graph: &LodGraph) -> Result<String, LodError> {
    let mut nodes = Vec::new();
    for t in normalized_triples(graph) {
        nodes.push(serde_json::json!({
            "@id": node_id(&t.subject),
            t.predicate.clone(): object_json(&t.object),
        }));
    }
    let value = serde_json::json!({"@context": graph.prefixes, "@graph": nodes});
    Ok(serde_json::to_string_pretty(&value)?)
}

fn triple_to_oxrdf_triple_ref(triple: &Triple) -> Result<TripleRef<'_>, LodError> {
    Ok(TripleRef::new(
        node_to_named_or_blank_ref(&triple.subject)?,
        NamedNodeRef::new(&triple.predicate).map_err(|e| LodError::RdfParsing(e.to_string()))?,
        node_to_term_ref(&triple.object)?,
    ))
}

fn triple_to_oxrdf_quad_ref<'a>(triple: &'a Triple, graph_name: GraphNameRef<'a>) -> Result<QuadRef<'a>, LodError> {
    Ok(QuadRef::new(
        node_to_named_or_blank_ref(&triple.subject)?,
        NamedNodeRef::new(&triple.predicate).map_err(|e| LodError::RdfParsing(e.to_string()))?,
        node_to_term_ref(&triple.object)?,
        graph_name,
    ))
}

fn node_to_named_or_blank_ref(node: &Node) -> Result<NamedOrBlankNodeRef<'_>, LodError> {
    match node {
        Node::Iri(iri) => Ok(NamedNodeRef::new(iri)
            .map_err(|e| LodError::RdfParsing(e.to_string()))?
            .into()),
        Node::Blank(id) => Ok(blank_node_ref(id)?.into()),
        Node::Literal { .. } => Err(LodError::RdfParsing("literal cannot be used as a subject".into())),
    }
}

fn node_to_term_ref(node: &Node) -> Result<TermRef<'_>, LodError> {
    match node {
        Node::Iri(iri) => Ok(NamedNodeRef::new(iri)
            .map_err(|e| LodError::RdfParsing(e.to_string()))?
            .into()),
        Node::Blank(id) => Ok(blank_node_ref(id)?.into()),
        Node::Literal { value, datatype, lang } => {
            if let Some(lang) = lang {
                Ok(LiteralRef::new_language_tagged_literal_unchecked(value, lang).into())
            } else if let Some(datatype) = datatype {
                let datatype = NamedNodeRef::new(datatype).map_err(|e| LodError::RdfParsing(e.to_string()))?;
                Ok(LiteralRef::new_typed_literal(value, datatype).into())
            } else {
                Ok(LiteralRef::new_simple_literal(value).into())
            }
        }
    }
}

fn blank_node_ref(id: &str) -> Result<BlankNodeRef<'_>, LodError> {
    let id = id.strip_prefix("_:").unwrap_or(id);
    BlankNodeRef::new(id).map_err(|e| LodError::RdfParsing(e.to_string()))
}

fn graph_name_to_ref(name: &str) -> Result<GraphNameRef<'_>, LodError> {
    if name.starts_with("_:") {
        Ok(blank_node_ref(name)?.into())
    } else {
        Ok(NamedNodeRef::new(name)
            .map_err(|e| LodError::RdfParsing(e.to_string()))?
            .into())
    }
}

fn node_id(n: &Node) -> String {
    match n {
        Node::Iri(i) => i.clone(),
        Node::Blank(b) => b.clone(),
        Node::Literal { value, .. } => value.clone(),
    }
}
fn object_json(n: &Node) -> Value {
    match n {
        Node::Iri(i) => serde_json::json!({"@id": i}),
        Node::Blank(b) => serde_json::json!({"@id": b}),
        Node::Literal { value, datatype, lang } => {
            let mut m = serde_json::Map::new();
            m.insert("@value".to_string(), Value::String(value.clone()));
            if let Some(dt) = datatype {
                m.insert("@type".to_string(), Value::String(dt.clone()));
            }
            if let Some(l) = lang {
                m.insert("@language".to_string(), Value::String(l.clone()));
            }
            Value::Object(m)
        }
    }
}

fn sorted_triples(graph: &LodGraph) -> Vec<Triple> {
    let mut triples = graph.all_triples().cloned().collect::<Vec<_>>();
    triples.sort();
    triples
}

fn normalized_triples(graph: &LodGraph) -> Vec<Triple> {
    let mut triples = sorted_triples(graph);
    let mut blank_map = BTreeMap::new();
    let mut blank_seq = 0usize;
    for triple in &mut triples {
        normalize_node(&mut triple.subject, &mut blank_map, &mut blank_seq);
        normalize_node(&mut triple.object, &mut blank_map, &mut blank_seq);
    }
    triples.dedup();
    triples
}

fn normalize_node(node: &mut Node, blank_map: &mut BTreeMap<String, String>, blank_seq: &mut usize) {
    match node {
        Node::Blank(id) => {
            let normalized = blank_map.entry(id.clone()).or_insert_with(|| {
                let next = format!("_:b{blank_seq}");
                *blank_seq += 1;
                next
            });
            *id = normalized.clone();
        }
        Node::Literal { .. } | Node::Iri(_) => {}
    }
}

fn parse_jsonld_subset(content: &str) -> Result<LodGraph, LodError> {
    let v: Value = serde_json::from_str(content)?;
    let mut graph = LodGraph::default();
    if let Some(ctx) = v.get("@context").and_then(|x| x.as_object()) {
        for (k, val) in ctx {
            if let Some(s) = val.as_str() {
                graph.prefixes.insert(k.clone(), s.to_string());
            }
        }
    }
    let arr = v
        .get("@graph")
        .and_then(|x| x.as_array())
        .ok_or_else(|| LodError::RdfParsing("JSON-LD subset expects @graph array".into()))?;
    for item in arr {
        let obj = item
            .as_object()
            .ok_or_else(|| LodError::RdfParsing("@graph item must be object".into()))?;
        let sid = obj
            .get("@id")
            .and_then(|x| x.as_str())
            .ok_or_else(|| LodError::RdfParsing("@graph item missing @id".into()))?;
        for (pred, val) in obj {
            if pred == "@id" {
                continue;
            }
            let object = if let Some(id) = val.get("@id").and_then(|x| x.as_str()) {
                Node::Iri(id.to_string())
            } else if let Some(lit) = val.get("@value").and_then(|x| x.as_str()) {
                Node::Literal {
                    value: lit.to_string(),
                    datatype: val.get("@type").and_then(|x| x.as_str()).map(String::from),
                    lang: val.get("@language").and_then(|x| x.as_str()).map(String::from),
                }
            } else if let Some(s) = val.as_str() {
                Node::Literal {
                    value: s.to_string(),
                    datatype: None,
                    lang: None,
                }
            } else {
                continue;
            };
            graph.triples.push(Triple {
                subject: Node::Iri(sid.to_string()),
                predicate: pred.clone(),
                object,
            });
        }
    }
    Ok(graph)
}
