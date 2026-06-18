# HTTP API

<!-- markdownlint-disable MD013 -->

The HTTP API mirrors the CLI and exposes the same core operations to the web
client or external tools.

## Endpoints

- `GET /api/health`
- `POST /api/inspect-text`
- `POST /api/validate-text`
- `POST /api/convert-text`
- `POST /api/visualize-text`

## Request Shapes

- inspection and validation requests send `content` plus an RDF `format`
- supported RDF formats include `turtle`, `n-triples`, `json-ld`, `rdf/xml`,
  and `trig`
- conversion requests send `content`, `from`, and `to`
- visualization requests send `content` plus an RDF `format`

## Example

```bash
curl -X POST http://127.0.0.1:8080/api/inspect-text \
  -H 'Content-Type: application/json' \
  -d '{"format":"turtle","content":"@prefix ex: <https://example.org/> .\
\nex:a ex:b \"c\" ."}'
```

The same endpoints also accept RDF/XML and TriG:

```bash
curl -X POST http://127.0.0.1:8080/api/inspect-text \
  -H 'Content-Type: application/json' \
  -d '{"format":"rdf/xml","content":"<?xml version=\"1.0\"?>\
\n<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\
\n</rdf:RDF>"}'
```

## Response Notes

- successful endpoints return JSON payloads
- validation errors are normalized into a small error object
- visualization responses include a graph payload and a JSON-LD payload for
  the frontend
- TriG datasets preserve named graphs in the shared core model
- RDF/XML serialization is available for single-graph RDF content

## Implementation Notes

- API routing and handlers live in `crates/lod-api/src/main.rs`
- all handlers call into the shared core crate
- CORS is enabled so the browser app can talk to the API during development

<!-- markdownlint-enable MD013 -->
