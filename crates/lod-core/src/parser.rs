use crate::{LodError, LodGraph, Node, RdfFormat, Triple};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn read_graph(path: impl AsRef<Path>, format: Option<RdfFormat>) -> Result<LodGraph, LodError> {
    let content = fs::read_to_string(&path)?;
    let fmt = match format {
        Some(f) => f,
        None => RdfFormat::from_path(path)?,
    };
    parse_graph(&content, fmt)
}

pub fn write_graph(graph: &LodGraph, path: impl AsRef<Path>, format: Option<RdfFormat>) -> Result<(), LodError> {
    let fmt = match format {
        Some(f) => f,
        None => RdfFormat::from_path(&path)?,
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
    }
}

pub fn serialize_graph(graph: &LodGraph, format: RdfFormat) -> Result<String, LodError> {
    match format {
        RdfFormat::Turtle => Ok(serialize_turtle(graph)),
        RdfFormat::NTriples => Ok(serialize_ntriples(graph)),
        RdfFormat::JsonLd => serialize_jsonld(graph),
    }
}

fn parse_line_based_rdf(content: &str) -> Result<LodGraph, LodError> {
    let mut graph = LodGraph::default();
    for (i, raw) in content.lines().enumerate() {
        let line_no = i + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("@prefix") || line.to_ascii_uppercase().starts_with("PREFIX") {
            parse_prefix(line, &mut graph.prefixes)
                .map_err(|e| LodError::RdfParsing(format!("line {line_no}: {e}")))?;
            continue;
        }
        let triple = parse_triple_line(line, &graph.prefixes)
            .map_err(|e| LodError::RdfParsing(format!("line {line_no}: {e}")))?;
        graph.triples.push(triple);
    }
    Ok(graph)
}

fn parse_turtle_rdf(content: &str) -> Result<LodGraph, LodError> {
    let mut graph = LodGraph::default();
    let mut blank_id_seq = 0usize;
    for (line_no, stmt) in split_turtle_statements(content) {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("@prefix") || trimmed.to_ascii_uppercase().starts_with("PREFIX") {
            parse_prefix(trimmed, &mut graph.prefixes)
                .map_err(|e| LodError::RdfParsing(format!("line {line_no}: {e}")))?;
            continue;
        }
        if trimmed.starts_with("@base") || trimmed.to_ascii_uppercase().starts_with("BASE") {
            parse_base(trimmed, &mut graph.base)
                .map_err(|e| LodError::RdfParsing(format!("line {line_no}: {e}")))?;
            continue;
        }
        let triples = parse_turtle_statement(trimmed, &graph.prefixes, graph.base.as_deref(), &mut blank_id_seq)
            .map_err(|e| LodError::RdfParsing(format!("line {line_no}: {e}")))?;
        graph.triples.extend(triples);
    }
    Ok(graph)
}

fn parse_prefix(line: &str, prefixes: &mut BTreeMap<String, String>) -> Result<(), String> {
    let cleaned = line.trim().trim_end_matches('.').trim();
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    if parts.len() < 3 {
        return Err("invalid prefix declaration".into());
    }
    if parts.len() > 3 {
        return Err("prefix declaration appears to be missing a `.` terminator".into());
    }
    let prefix = parts[1].trim_end_matches(':').to_string();
    let iri = trim_iri(parts[2]).ok_or("invalid prefix IRI")?.to_string();
    prefixes.insert(prefix, iri);
    Ok(())
}

fn parse_base(line: &str, base: &mut Option<String>) -> Result<(), String> {
    let cleaned = line.trim().trim_end_matches('.').trim();
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("invalid base declaration".into());
    }
    if parts.len() > 2 {
        return Err("base declaration appears to be missing a `.` terminator".into());
    }
    let iri = trim_iri(parts[1]).ok_or("invalid base IRI")?.to_string();
    *base = Some(iri);
    Ok(())
}

fn parse_triple_line(line: &str, prefixes: &BTreeMap<String, String>) -> Result<Triple, String> {
    let line = line.trim_end_matches('.').trim();
    let (s, rest) = take_token(line).ok_or("missing subject")?;
    let (p, rest) = take_token(rest.trim()).ok_or("missing predicate")?;
    let o = rest.trim();
    if o.is_empty() {
        return Err("missing object".into());
    }
    Ok(Triple {
        subject: parse_subject(s, prefixes, None)?,
        predicate: expand_term(p, prefixes, None)?,
        object: parse_object_simple(o, prefixes)?,
    })
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
        parse_predicate_object_list(subject.clone(), clause.trim(), prefixes, base, blank_id_seq, &mut triples)?;
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
    let text = trim_trailing_terminators(text);
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
            c if !in_string
                && !in_iri
                && depth == 0
                && (c.is_whitespace() || matches!(c, ';' | ',' | '.')) =>
            {
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
    let o = trim_trailing_terminators(o);
    if contains_bare_whitespace(o) && !o.starts_with('<') && !o.starts_with('"') && !o.starts_with('[') && !o.starts_with('(') && !o.starts_with("_:") {
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

fn parse_object_simple(o: &str, prefixes: &BTreeMap<String, String>) -> Result<Node, String> {
    let o = trim_trailing_terminators(o);
    if contains_bare_whitespace(o) && !o.starts_with('<') && !o.starts_with('"') && !o.starts_with("_:") {
        return Err("unexpected whitespace in object".into());
    }
    if o.starts_with('"') {
        parse_literal(o, prefixes, None)
    } else if o.starts_with("_:") {
        Ok(Node::Blank(o.to_string()))
    } else {
        Ok(Node::Iri(expand_term(o, prefixes, None)?))
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
    let subject = Node::Blank(fresh_blank_id(blank_id_seq));
    let clauses = split_top_level(inner.trim(), ';');
    let mut saw_predicate = false;
    for clause in clauses {
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
    let mut items = Vec::new();
    let mut cursor = inner.trim();
    while !cursor.trim().is_empty() {
        let (frag, next) = take_object_fragment(cursor).ok_or("malformed RDF collection")?;
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
            c if !in_string
                && !in_iri
                && depth == 0
                && (c.is_whitespace() || matches!(c, ';' | ',' | '.')) =>
            {
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
    if iri.starts_with("http://") || iri.starts_with("https://") || iri.starts_with("urn:") || iri.starts_with("mailto:") {
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
    for t in sorted_triples(graph) {
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
    sorted_triples(graph)
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
    for t in sorted_triples(graph) {
        nodes.push(serde_json::json!({
            "@id": node_id(&t.subject),
            t.predicate.clone(): object_json(&t.object),
        }));
    }
    let value = serde_json::json!({"@context": graph.prefixes, "@graph": nodes});
    Ok(serde_json::to_string_pretty(&value)?)
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
    let mut triples = graph.triples.clone();
    triples.sort();
    triples
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
