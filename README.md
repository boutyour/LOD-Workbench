# LOD Workbench RS

<!-- markdownlint-disable MD013 -->
[![CI](https://github.com/boutyour/LOD-Workbench/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/boutyour/LOD-Workbench/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**LOD Workbench: A Multi-Utility Toolkit for Linked Open Data Engineering**
implemented primarily in Rust.

This repository contains a beta-ready RDF workbench with:

- a Rust core library: `lod-core`
- a CLI interface: `lod`
- a Web API: `lod-api` using Axum
- a React/Vite web UI
- sample RDF, CSV, YAML, SHACL, and HTML report assets

The project is organized around a practical file-based workflow and focuses on
inspection, syntax validation, SHACL validation, conversion, visualization,
and CSV-to-RDF mapping. The beta already includes live editing in the web app,
separate syntax-only and SHACL validation views, richer sample datasets, and
downloadable reports for common workflows.

## Beta 4 Supported Features

`v0.1.0-beta.4` focuses on a tighter editing and validation workflow:

- live refresh while editing RDF or SHACL text
- save back in the selected input format
- syntax-only validation for the input graph and loaded shapes
- dedicated SHACL validation tab and report view
- improved RDF graph inspection for data and shapes
- clearer validation diagnostics and report output
- more readable graph export and SVG output
- adjustable editors for RDF and SHACL content
- sample datasets that exercise lists, bags, blank nodes, and typed literals
- RDF/XML import/export
- TriG import/export with named graphs

For a more structured reference guide, see [docs/README.md](docs/README.md).

## At A Glance

- Core RDF processing library for inspection, validation, SHACL reporting,
  conversion, mapping, and visualization
- Command-line interface for local workflows
- Web API for integrating RDF features into other tools
- React/Vite web client with live editing, split validation views, and
  visualization
- GitHub Actions automation for Rust checks and web builds

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
  validate    Validate RDF syntax
  shacl       Validate RDF against SHACL shapes
  convert     Convert RDF output
  map         Map CSV to RDF
  visualize   Visualize RDF graph
  help        Print help
```

```text
$ cargo test -p lod-core --tests
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Contents

- [LOD Workbench RS](#lod-workbench-rs)
  - [Beta 4 Supported Features](#beta-4-supported-features)
  - [At A Glance](#at-a-glance)
  - [Quick Start](#quick-start)
  - [Screenshots](#screenshots)
  - [Command Line Output](#command-line-output)
  - [Contents](#contents)
  - [Repository structure](#repository-structure)
  - [Requirements](#requirements)
  - [Build](#build)
  - [Features](#features)
    - [Core RDF workflow](#core-rdf-workflow)
    - [Beta 4 feature set](#beta-4-feature-set)
    - [Supported RDF syntax](#supported-rdf-syntax)
    - [Web application](#web-application)
    - [API](#api)
    - [Tooling and release](#tooling-and-release)
  - [CLI usage](#cli-usage)
    - [Inspect RDF](#inspect-rdf)
    - [Convert RDF](#convert-rdf)
    - [Validate RDF syntax and IRI quality](#validate-rdf-syntax-and-iri-quality)
    - [Map CSV to RDF](#map-csv-to-rdf)
    - [Visualize RDF as HTML](#visualize-rdf-as-html)
  - [API usage](#api-usage)
  - [Web interface](#web-interface)
  - [Enhanced test coverage](#enhanced-test-coverage)
  - [Design patterns used](#design-patterns-used)
  - [Roadmap](#roadmap)
  - [Current Status](#current-status)
  - [Project Docs](#project-docs)
  - [Author \& Contact](#author--contact)

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
├── .github/workflows/ # single CI pipeline (GitHub Actions)
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
- Inspect input data and loaded SHACL shapes side by side
- Validate syntax and IRI quality
- Validate RDF data against SHACL shapes
- Convert between Turtle, N-Triples, RDF/XML, TriG, and JSON-LD
- Map CSV data to RDF
- Render RDF graphs for browser viewing or SVG export
- Open, edit, and save RDF content directly from the web editor
- Generate syntax and SHACL reports in JSON, text, or HTML

### Beta 4 feature set

- live validation and preview refresh while typing
- explicit syntax-only validation tab
- dedicated SHACL report tab
- SHACL shape editing in the web UI
- improved inspection for RDF input and loaded shapes
- adjustable editor panels
- richer sample RDF fixtures for testing RDF syntax features
- RDF/XML support for legacy RDF content
- TriG support for multi-graph datasets and named graphs

### Supported RDF syntax

- Prefixes, base IRIs, and `a` shorthand
- Typed and language-tagged literals, including escaped strings
- Blank nodes, blank node property lists, RDF lists, and RDF bags
- Comments, multiline statements, and round-trip serialization for the supported subset

### Web application

- Live text editing with open/save support
- Separate Inspect, Validate, SHACL, Convert, and Visualize tabs
- Syntax-only validation in one tab and SHACL reporting in another
- Graph pan, zoom, drag, and SVG export
- Adjustable RDF and SHACL editors with real-time refresh
- Responsive layout with clear loading and error states

### API

- `GET /api/health`
- `POST /api/inspect-text`
- `POST /api/validate-text`
- `POST /api/validate-text-detail`
- `POST /api/convert-text`
- `POST /api/visualize-text`

### Tooling and release

- GitHub Actions CI for formatting, linting, build, and tests
- Manual release tagging when you want to publish a snapshot

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

By default, validation performs syntax and IRI checks. Build with the optional
`lod-core/rudof-shacl` feature to enable real SHACL constraint validation
through Rudof:

```bash
cargo run -p lod --features lod-core/rudof-shacl -- validate \
  examples/data.ttl examples/shapes.ttl
```

For a dedicated SHACL report from the CLI, use the `shacl` subcommand:

```bash
cargo run -p lod -- shacl examples/data.ttl examples/shapes.ttl
```

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
- one editor for SHACL shapes
- one tab per task: inspect, validate, SHACL, convert, and visualize
- syntax-only validation in one tab and SHACL constraint reporting in another
- a graph view with drag, zoom, and SVG export
- save/download actions for the current text and converted output
- immediate refresh while typing
- responsive behavior for desktop and mobile screens

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

The beta-to-release plan lives in [docs/roadmap.md](docs/roadmap.md).
Each version entry there includes the features supported or planned for that
release line, so you can quickly compare what changes from beta to beta and
from beta to `v1.0.0`.

---

## Current Status

- Core library: stable for the supported RDF subset
- CLI: functional with inspect, validate, SHACL, convert, map, and visualize commands
- API: functional with separate syntax validation and SHACL report endpoints
- Web UI: functional, beta-ready, and live-updating
- Tests: passing
- Release readiness: beta

---


## Project Docs

The dedicated documentation hub lives in [docs/README.md](docs/README.md) and
links to the focused pages below:

- [Architecture](docs/architecture.md)
- [CLI Reference](docs/cli.md)
- [HTTP API](docs/api.md)
- [Web UI](docs/web.md)
- [Roadmap](docs/roadmap.md)
- [Release](docs/release.md)
- [Branch Protection](docs/branch-protection.md)
- [Troubleshooting](docs/troubleshooting.md)

It also keeps the screenshots and general overview in one place.

<!-- markdownlint-enable MD013 -->

---

## Author & Contact

Created by Pr. Youness BOUTYOUR, ENSIAS, Mohammed V University.

Email: `youness.boutyour@um5.ac.ma`
