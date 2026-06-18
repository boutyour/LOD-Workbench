# CLI Reference

<!-- markdownlint-disable MD013 -->

The CLI is the quickest way to run the toolkit locally.

## Command Summary

- `inspect` reads RDF and prints summary metrics
- `validate` checks syntax, IRI quality, and optional SHACL constraints
- `convert` transforms RDF between supported formats, including Turtle,
  N-Triples, RDF/XML, TriG, and JSON-LD
- `map` turns CSV rows into RDF triples
- `visualize` writes an HTML visualization report

## Examples

Inspect a Turtle file:

```bash
cargo run -p lod -- inspect examples/data.ttl
```

Inspect RDF/XML or TriG:

```bash
cargo run -p lod -- inspect examples/transport.rdf
cargo run -p lod -- inspect examples/transport.trig
```

Validate a graph and write a report:

```bash
cargo run -p lod -- validate examples/data.ttl examples/shapes.ttl \
  --report reports/report.html
```

Run validation with Rudof-powered SHACL constraints:

```bash
cargo run -p lod --features lod-core/rudof-shacl -- validate \
  examples/data.ttl examples/shapes.ttl
```

Convert RDF into N-Triples:

```bash
cargo run -p lod -- convert examples/data.ttl /tmp/data.nt --from turtle \
  --to n-triples
```

Convert into RDF/XML or TriG:

```bash
cargo run -p lod -- convert examples/data.ttl /tmp/data.rdf --from turtle \
  --to rdf/xml
cargo run -p lod -- convert examples/data.ttl /tmp/data.trig --from turtle \
  --to trig
```

Map CSV to RDF:

```bash
cargo run -p lod -- map examples/researchers.csv examples/mapping.yml \
  /tmp/researchers.ttl --to turtle
```

Visualize RDF as HTML:

```bash
cargo run -p lod -- visualize examples/data.ttl --output reports/graph.html
```

## Output Style

The CLI prints compact human-readable summaries so it can be used directly in
shell scripts or by people exploring the project by hand.

## Implementation Notes

- CLI argument parsing lives in `crates/lod-cli/src/main.rs`
- The actual work is delegated to `LodWorkbench`
- Format detection and conversion logic stay in the core crate

<!-- markdownlint-enable MD013 -->
