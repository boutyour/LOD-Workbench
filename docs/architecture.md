# Architecture

LOD Workbench is organized as a small layered workspace:

- `crates/lod-core` contains the parser, model, and shared services
- `crates/lod-cli` exposes the command-line interface
- `crates/lod-api` exposes the same capabilities over HTTP
- `apps/web` renders the React/Vite browser UI

## Core Flow

1. RDF text is loaded from a file, the API, or the editor.
2. The parser builds an in-memory `LodGraph`.
3. Inspection, validation, conversion, mapping, and visualization reuse that graph.
4. The API returns structured JSON responses.
5. The frontend turns the graph payload into an interactive SVG view.

## Main Types

- `Node`, `Triple`, and `LodGraph` define the RDF model
- `InspectionReport` and `ValidationReport` carry analysis results
- `VisualizationGraph` carries browser-friendly graph data
- `LodWorkbench` is the façade that bundles the core services together

## Design Notes

- The implementation stays intentionally compact so the codebase is easy to
  read and extend.
- The services are separated by responsibility, which keeps the CLI, API, and
  web UI thin.
- The graph visualizer is browser-native SVG so it stays lightweight and
  exportable.

## Where To Look

- Parser and serializer logic: `crates/lod-core/src/parser.rs`
- Shared model types: `crates/lod-core/src/model.rs`
- Facade and service entry points: `crates/lod-core/src/facade.rs`
- API handlers: `crates/lod-api/src/main.rs`
- Graph renderer: `apps/web/src/components/GraphViewer.jsx`
