# Roadmap

This roadmap focuses on the path from the current beta line to `v1.0.0`.

## `v0.1.0-beta.4`

Target focus:

- keep the editor, inspect, validate, convert, and visualize tabs in sync
- save the current text in the selected input format
- improve parser coverage for the supported Turtle subset
- keep blank nodes, lists, and bags readable in the graph view
- make the graph preview and SVG export more reliable
- keep beta preview builds stable on GitHub Actions

## `v0.1.0-beta.5`

Target focus:

- reduce noisy warnings and CI friction
- improve error messages for parsing and validation
- harden open/save flows for edited text
- tighten the docs for setup, troubleshooting, and release steps
- expand regression tests for parser and service behavior

## `v0.1.0-rc.1`

Target focus:

- freeze user-facing behavior
- fix only blockers, regressions, and data-loss bugs
- verify release packaging and preview availability
- complete the final documentation pass
- confirm the CLI, API, and web UI behave consistently

## `v1.0.0`

Target focus:

- publish a stable tag
- treat the supported RDF subset as the documented contract
- keep the CLI, API, and web UI aligned
- ship the preview workflow as a stable release process
- leave room for future RDF and SHACL adapter work

## After `v1`

Future versions can focus on larger RDF features without disturbing the
stable `v1` line:

- fuller RDF parser adapters
- complete SHACL integration
- RDF/XML and richer JSON-LD support
- RDF diff and normalization tooling
- longer-running web or workspace features
