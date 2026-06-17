# CLI Reference

The CLI is the quickest way to run the toolkit locally.

## Command Summary

- `inspect` reads RDF and prints summary metrics
- `validate` checks syntax and IRI quality
- `convert` transforms RDF between supported formats
- `map` turns CSV rows into RDF triples
- `visualize` writes an HTML visualization report

## Examples

Inspect a Turtle file:

```bash
cargo run -p lod -- inspect examples/data.ttl
```

Validate a graph and write a report:

```bash
cargo run -p lod -- validate examples/data.ttl examples/shapes.ttl --report reports/report.html
```

Convert RDF into N-Triples:

```bash
cargo run -p lod -- convert examples/data.ttl /tmp/data.nt --from turtle --to n-triples
```

Map CSV to RDF:

```bash
cargo run -p lod -- map examples/researchers.csv examples/mapping.yml /tmp/researchers.ttl --to turtle
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
