# LOD Workbench Documentation

This is the entry point for the project docs. Use the topic pages below for the
main reference material, then return here for screenshots, release notes, and
quick navigation.

## Pages

- [Architecture](architecture.md)
- [CLI Reference](cli.md)
- [HTTP API](api.md)

## Project Overview

LOD Workbench is a Linked Open Data toolkit that can inspect, validate,
convert, map, and visualize RDF data. The same core logic is shared by the
Rust CLI, the HTTP API, and the web UI.

## Screenshots

| Web UI | Graph | CLI |
| --- | --- | --- |
| [![Web UI](screenshots/web-ui.svg)](screenshots/web-ui.svg) | [![Graph](screenshots/graph-preview.svg)](screenshots/graph-preview.svg) | [![CLI](screenshots/cli-output.svg)](screenshots/cli-output.svg) |

## Release Flow

- `beta` branch pushes deploy the web client to GitHub Pages
- version tags such as `v0.1.0-beta.2` publish GitHub Releases
- the release workflow packages the Rust binaries and web bundle

## Troubleshooting

- If the UI says the API is unreachable, make sure `lod-api` is running on
  `127.0.0.1:8080`.
- If validation fails on a sample, check that the RDF syntax matches the
  supported subset.
- If graph export looks clipped, use the SVG export button in the visualizer
  to download the full graph bounds.
