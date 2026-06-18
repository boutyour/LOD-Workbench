# Roadmap

This roadmap focuses on the beta series and the path to `v1.0.0`.

## `v0.1.0-beta.3`

Current release line:

- keep the single CI workflow green
- keep a simple preview build available
- stabilize the parser on the supported RDF subset
- keep the web UI responsive, readable, and predictable
- make the editor save/open flow reliable
- keep the documentation aligned with the shipped behavior

## `v0.1.0-beta.4`

Target focus:

- stabilize the auto-refresh editor flow on text changes
- keep saving in the selected input format reliable
- polish parser coverage for the supported Turtle subset
- keep lists and bags readable in the graph view
- make validation errors clearer and easier to act on
- make graph export more readable and reliable
- keep beta preview builds stable on GitHub Actions

## `v0.1.0-beta.5`

Target focus:

- reduce noisy warnings and CI friction
- harden open/save flows for edited text
- tighten the docs for setup, troubleshooting, and release steps
- expand regression tests for parser, validation, and service behavior

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
- ship the beta-tested packaging and preview workflow as stable
- leave room for future RDF and SHACL adapter work

## After `v1`

Future versions can focus on larger RDF features without disturbing the
stable `v1` line:

- fuller RDF parser adapters
- complete SHACL integration
- RDF/XML and richer JSON-LD support
- RDF diff and normalization tooling
- longer-running web or workspace features
