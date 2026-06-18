# Web UI

<!-- markdownlint-disable MD013 -->

The web application provides the fastest interactive way to explore RDF data.

## What It Includes

- live text editing with automatic backend refresh
- inspect, validate, convert, and visualize tabs
- save the current editor text in the selected input format
- open RDF files from disk
- graph pan, zoom, drag, and SVG export
- responsive layout for desktop and mobile screens

## How To Use It

1. Start the API with `cargo run -p lod-api`.
2. Start the frontend with `cd apps/web && npm install && npm run dev`.
3. Open <http://127.0.0.1:5173>.
4. Paste RDF in the editor or load a sample file.
5. Switch tabs to inspect, validate, convert, and visualize the graph.

## Graph Viewer Notes

- the graph is rendered as SVG, so browser zoom and export stay lightweight
- node dragging is available directly in the graph canvas
- the export button saves the full graph bounds instead of only the visible
  viewport

## Screenshots

| Web UI | Graph |
| --- | --- |
| [![Web UI](screenshots/web-ui.svg)](screenshots/web-ui.svg) | [![Graph](screenshots/graph-preview.svg)](screenshots/graph-preview.svg) |

## Implementation Notes

- the React entry point lives in `apps/web/src/main.jsx`
- the graph renderer lives in `apps/web/src/components/GraphViewer.jsx`
- the API helper lives in `apps/web/src/lib/api.js`
- styling lives in `apps/web/src/style.css`

<!-- markdownlint-enable MD013 -->
