# LOD Workbench RS

**LOD Workbench: A Multi-Utility Toolkit for Linked Open Data Engineering** implemented primarily in Rust.

Created by Pr. Youness BOUTYOUR, ENSIAS, Mohammed V University.
Contact: `youness.boutyour@um5.ac.ma`.

This repository contains a beta-ready RDF workbench with:

- a Rust core library: `lod-core`
- a CLI interface: `lod`
- a Web API: `lod-api` using Axum
- a React/Vite web UI
- sample RDF, CSV, YAML, and HTML report assets

The project is organized around a practical file-based workflow and supports a useful subset of Turtle, N-Triples, and JSON-LD for education, prototyping, and tool architecture demonstration. Full W3C RDF parsing and full SHACL validation can be integrated later through dedicated adapters.

---

## Repository structure

```text
lod-workbench-rs/
├── crates/
│   ├── lod-core/      # shared domain and services
│   ├── lod-cli/       # command-line interface binary: lod
│   └── lod-api/       # Axum web API
├── apps/
│   └── web/           # React/Vite interface
├── examples/          # sample Turtle, CSV, YAML mapping, SHACL skeleton
├── reports/           # output directory for validation/visualization reports
├── scripts/           # helper scripts
├── tests/             # integration test data
├── .github/workflows/ # CI pipeline (GitHub Actions)
├── rustfmt.toml       # Rust formatting configuration
├── Makefile
└── README.md
```

---

## Requirements

- Rust stable toolchain (edition 2021)
- Cargo
- Node.js and npm, only for the React web interface

---

## Build

```bash
cargo build --workspace
```

Run tests:

```bash
cargo test --workspace
```

Run lints (clippy + formatting):

```bash
make check    # clippy only
make fmt      # format code
make lint     # format + clippy
```

---

## Features

### Core RDF workflow

- RDF inspection from files or text input
- RDF validation with syntax checks and IRI quality checks
- RDF conversion between Turtle, N-Triples, and JSON-LD
- CSV to RDF mapping
- RDF visualization to HTML/SVG-ready graph output

### Supported RDF syntax

- Turtle prefixes and base IRI handling
- `a` shorthand for `rdf:type`
- typed literals
- language-tagged literals
- escaped literals
- blank nodes
- blank node property lists
- RDF collections / lists
- RDF bag examples
- comments and multiline statements
- round-trip serialization for the supported subset

### Web application

- Inspect tab with summary metrics and distributions
- Validate tab with readable issue reporting
- Convert tab with format selection and output preview
- Visualize tab with graph rendering, pan and zoom, drag support, and SVG export
- Live processing while typing in the editor
- Save edited text in the selected input format
- Load RDF files from disk
- Responsive layout for desktop and mobile screens
- Clear loading and error states

### API

- `GET /api/health`
- `POST /api/inspect-text`
- `POST /api/validate-text`
- `POST /api/convert-text`
- `POST /api/visualize-text`

### Tooling and release

- GitHub Actions CI for formatting, linting, build, and tests
- GitHub Pages beta deployment for the web client
- Example data files and generated HTML reports
- Rust workspace packaging for shared core services

---

## CLI usage

### Inspect RDF

```bash
cargo run -p lod -- inspect examples/data.ttl
```

### Convert RDF

```bash
cargo run -p lod -- convert examples/data.ttl /tmp/data.nt --from turtle --to n-triples
cargo run -p lod -- convert examples/data.ttl /tmp/data.jsonld --from turtle --to json-ld
```

### Validate RDF syntax and IRI quality

```bash
cargo run -p lod -- validate examples/data.ttl examples/shapes.ttl --report reports/report.html
```

V1 performs syntax validation, IRI checks, and a placeholder notice for SHACL. Full SHACL validation is planned for V1.1 through a dedicated adapter.

### Map CSV to RDF

```bash
cargo run -p lod -- map examples/researchers.csv examples/mapping.yml /tmp/researchers.ttl --to turtle
```

### Visualize RDF as HTML

```bash
cargo run -p lod -- visualize examples/data.ttl --output reports/graph.html
```

Then open `reports/graph.html` in a browser.

---

## API usage

Start the API:

```bash
cargo run -p lod-api
```

Health check:

```bash
curl http://127.0.0.1:8080/api/health
```

Inspect text RDF:

```bash
curl -X POST http://127.0.0.1:8080/api/inspect-text \
  -H 'Content-Type: application/json' \
  -d '{"format":"turtle","content":"@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" ."}'
```

Convert text RDF:

```bash
curl -X POST http://127.0.0.1:8080/api/convert-text \
  -H 'Content-Type: application/json' \
  -d '{"from":"turtle","to":"json-ld","content":"@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" ."}'
```

---

## Web interface

Start the Rust API first:

```bash
cargo run -p lod-api
```

Then start the React app:

```bash
cd apps/web
npm install
npm run dev
```

Open <http://127.0.0.1:5173> in your browser. The web interface provides:

- **Inspect** — view triple counts, subject/predicate/object breakdown, class and property distributions
- **Validate** — check RDF syntax and IRI quality
- **Convert** — between Turtle, N-Triples, and JSON-LD
- **Preview** — JSON-LD graph payload
- **Error handling** — clear error messages when the API is unreachable or returns errors
- **Loading states** — visual feedback during API calls
- **Responsive layout** — adapts to mobile and desktop screens

### Beta deployment on GitHub Pages

The web client can be published as a beta build from the `beta` branch using GitHub Pages.

1. Set the repository variable or secret `VITE_API_URL` to the deployed `lod-api` base URL.
2. Push the branch `beta`, or run the `Beta Pages` workflow manually from GitHub Actions.
3. GitHub Pages will serve the built UI from `apps/web/dist`.

The beta site is the frontend only. It still needs a reachable API endpoint for inspect, validate, convert, and visualize actions.

### Release workflow

Use semantic version tags for published snapshots:

- `v0.1.0-beta.2` for beta builds
- `v0.1.0-rc.1` for release candidates
- `v0.1.0` for stable releases

Release checklist:

1. Merge the target changes into `beta` and verify the beta Pages deployment.
2. Run the workspace tests locally or in CI.
3. Create a tag such as `v0.1.0-beta.2`.
4. Push the tag to GitHub.
5. The `Release` workflow builds the Rust binaries and web client, then publishes a GitHub Release with downloadable archives.

---

## Enhanced test coverage

Test suite: **33 tests** (up from 2) covering:

| Category | Tests |
|---|---|
| Turtle parsing (prefixes, triples, blank nodes, `a` shorthand, typed literals, lang tags, escaped literals, comments) | 12 |
| N-Triples parsing | 2 |
| JSON-LD parsing | 1 |
| Serialization (Turtle, N-Triples, JSON-LD, lang-tagged, typed) | 5 |
| Round-trip (parse → serialize → parse) | 2 |
| Format detection (extensions, case-insensitive) | 2 |
| Edge cases (empty, comments only, invalid syntax, unknown format) | 4 |
| Model equality and ordering | 2 |
| Service integration (construct, error on missing file, bad format) | 7 |

---

## Design patterns used

The Rust implementation follows pattern-equivalent idioms:

- **Facade**: `LodWorkbench` exposes a simple API to CLI and Web.
- **Adapter**: parser/writer functions isolate the RDF processing layer.
- **Strategy-like services**: `ConversionService`, `InspectionService`, `ValidationService`, `MappingService`, `VisualizationService`.
- **Builder-like output generation**: HTML reports and visualization pages.
- **DTOs**: request and report structs in `model.rs`.
- **Result-based error handling**: centralized `LodError` enum.

---

## Roadmap

### V1.1

- integrate a full RDF parser adapter, preferably Sophia or Oxigraph
- integrate full SHACL through Rudof or another SHACL engine
- add RDF/XML and complete JSON-LD support
- add RDF diff and normalization commands

### V2

- job-based web processing
- upload/download file management
- persistent project workspace
- SPARQL support
- quality scoring
- repair assistant
- plugin registry

---

## Current Status

- Core library: stable for the supported RDF subset
- CLI: functional
- API: functional
- Web UI: functional and beta-deployable
- Tests: passing
- Release readiness: beta

---

## Important limitations

This first version intentionally implements a compact RDF parser to keep the repository self-contained and understandable. It is suitable for controlled Turtle/N-Triples examples and educational workflows. For production-grade RDF/JSON-LD/SHACL compliance, replace the parser and validator internals with adapters to mature RDF crates.
