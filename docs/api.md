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
- conversion requests send `content`, `from`, and `to`
- visualization requests send `content` plus an RDF `format`

## Example

```bash
curl -X POST http://127.0.0.1:8080/api/inspect-text \
  -H 'Content-Type: application/json' \
  -d '{"format":"turtle","content":"@prefix ex: <https://example.org/> .\
\nex:a ex:b \"c\" ."}'
```

## Response Notes

- successful endpoints return JSON payloads
- validation errors are normalized into a small error object
- visualization responses include a graph payload and a JSON-LD payload for
  the frontend

## Implementation Notes

- API routing and handlers live in `crates/lod-api/src/main.rs`
- all handlers call into the shared core crate
- CORS is enabled so the browser app can talk to the API during development

<!-- markdownlint-enable MD013 -->
