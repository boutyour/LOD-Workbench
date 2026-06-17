# LOD Workbench RS

<!-- markdownlint-disable MD013 -->
[![CI](https://github.com/boutyour/LOD-Workbench/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/boutyour/LOD-Workbench/actions/workflows/ci.yml)
[![Beta deploy](https://github.com/boutyour/LOD-Workbench/actions/workflows/beta-pages.yml/badge.svg?branch=beta)](https://github.com/boutyour/LOD-Workbench/actions/workflows/beta-pages.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**LOD Workbench: A Multi-Utility Toolkit for Linked Open Data Engineering**
implemented primarily in Rust.

This repository contains a beta-ready RDF workbench with:

- a Rust core library: `lod-core`
- a CLI interface: `lod`
- a Web API: `lod-api` using Axum
- a React/Vite web UI
- sample RDF, CSV, YAML, and HTML report assets

The project is organized around a practical file-based workflow and supports a
useful subset of Turtle, N-Triples, and JSON-LD for education, prototyping, and
tool architecture demonstration. Full W3C RDF parsing and full SHACL
validation can be integrated later through dedicated adapters.

For a more structured reference guide, see [docs/README.md](docs/README.md).

## At A Glance

- Core RDF processing library for inspection, validation, conversion, mapping,
  and visualization
- Command-line interface for local workflows
- Web API for integrating RDF features into other tools
- React/Vite web client with live editing and visualization
- GitHub Actions automation for CI, beta deploys, and releases

## Quick Start

```bash
git clone https://github.com/boutyour/LOD-Workbench.git
cd LOD-Workbench
cargo build --workspace
cargo run -p lod-api
cd apps/web
npm install
npm run dev
```

Open the web app at <http://127.0.0.1:5173>. The API should be available at <http://127.0.0.1:8080>.

## Screenshots

| Web UI preview | Graph preview | CLI preview |
| --- | --- | --- |
| [![Web UI preview](docs/screenshots/web-ui.svg)](docs/screenshots/web-ui.svg) | [![Graph preview](docs/screenshots/graph-preview.svg)](docs/screenshots/graph-preview.svg) | [![CLI preview](docs/screenshots/cli-output.svg)](docs/screenshots/cli-output.svg) |

## Command Line Output

```text
$ cargo run -p lod -- --help
lod 0.1.0
Multi-utility toolkit for Linked Open Data engineering

Usage:
  lod [OPTIONS] <COMMAND>

Commands:
  inspect     Inspect RDF input
  validate    Validate RDF input
  convert     Convert RDF output
  map         Map CSV to RDF
  visualize   Visualize RDF graph
  help        Print help
```

```text
$ cargo test -p lod-core --tests
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Contents

- [Repository structure](#repository-structure)
- [Requirements](#requirements)
- [Build](#build)
- [Features](#features)
- [CLI usage](#cli-usage)
- [API usage](#api-usage)
- [Web interface](#web-interface)
- [Beta deployment on GitHub Pages](#beta-deployment-on-github-pages)
- [Release workflow](#release-workflow)
- [Enhanced test coverage](#enhanced-test-coverage)
- [Design patterns used](#design-patterns-used)
- [Roadmap](#roadmap)
- [Current Status](#current-status)
- [Important limitations](#important-limitations)
- [Author & Contact](#author--contact)
- [Project Docs](#project-docs)

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

- Inspect RDF from files or pasted text
- Validate syntax and IRI quality
- Convert between Turtle, N-Triples, and JSON-LD
- Map CSV data to RDF
- Render RDF graphs for browser viewing or SVG export

### Supported RDF syntax

- Prefixes, base IRIs, and `a` shorthand
- Typed and language-tagged literals, including escaped strings
- Blank nodes, blank node property lists, RDF lists, and RDF bags
- Comments, multiline statements, and round-trip serialization for the supported subset

### Web application

- Live text editing with open/save support
- Inspect, validate, convert, and visualize tabs
- Graph pan, zoom, drag, and SVG export
- Responsive layout with clear loading and error states

### API

- `GET /api/health`
- `POST /api/inspect-text`
- `POST /api/validate-text`
- `POST /api/convert-text`
- `POST /api/visualize-text`

### Tooling and release

- GitHub Actions CI for formatting, linting, build, and tests
- GitHub Pages beta deployment from the `beta` branch
- Tagged GitHub Releases for beta, release candidate, and stable snapshots

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

## API usage

Start the API:

```bash
cargo run -p lod-api
```

Quick checks:

```bash
curl http://127.0.0.1:8080/api/health
curl -X POST http://127.0.0.1:8080/api/inspect-text \
  -H 'Content-Type: application/json' \
  -d '{"format":"turtle","content":"@prefix ex: <https://example.org/> .\nex:a ex:b \"c\" ."}'
```

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

Open <http://127.0.0.1:5173> in your browser. The web interface focuses on:

- one editor for live RDF input
- one tab per task: inspect, validate, convert, and visualize
- a graph view with drag, zoom, and SVG export
- save/download actions for the current text and converted output
- responsive behavior for desktop and mobile screens

### Beta deployment on GitHub Pages

The web client publishes automatically from the `beta` branch through GitHub
Pages. Set `VITE_API_URL` to the deployed API endpoint, then push to `beta` or
run the `Beta Pages` workflow manually. The beta site is frontend-only and
still needs a reachable API for inspect, validate, convert, and visualize
actions.

### Release workflow

Use semantic version tags for published snapshots:

- `v0.1.0-beta.2` for beta builds
- `v0.1.0-rc.1` for release candidates
- `v0.1.0` for stable releases

Release checklist:

1. Merge target changes into `beta` and verify the beta Pages deployment.
2. Run the workspace tests locally or in CI.
3. Create a tag such as `v0.1.0-beta.2`.
4. Push the tag to GitHub.
5. The `Release` workflow builds the Rust binaries and web client, then
   publishes a GitHub Release with downloadable archives.

---

## Enhanced test coverage

Test suite: **33 tests** (up from 2) covering:

| Category | Tests |
| --- | --- |
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
- **Strategy-like services**: `ConversionService`, `InspectionService`,
  `ValidationService`, `MappingService`, `VisualizationService`.
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

## Project Docs

The dedicated documentation hub lives in [docs/README.md](docs/README.md) and
links to the focused pages below:

- [Architecture](docs/architecture.md)
- [CLI Reference](docs/cli.md)
- [HTTP API](docs/api.md)
- [Web UI](docs/web.md)
- [Release](docs/release.md)
- [Branch Protection](docs/branch-protection.md)
- [Troubleshooting](docs/troubleshooting.md)

It also keeps the screenshots and general overview in one place.

<!-- markdownlint-enable MD013 -->

---

## Author & Contact

Created by Pr. Youness BOUTYOUR, ENSIAS, Mohammed V University.

Email: `youness.boutyour@um5.ac.ma`
