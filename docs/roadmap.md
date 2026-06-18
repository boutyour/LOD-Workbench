# Roadmap

This roadmap covers the beta series and the path to `v1.0.0`.

## Current Beta

Current release line: `v0.1.0-beta.3`

- keep the single CI workflow green
- keep the GitHub Pages preview available
- stabilize the parser on the supported RDF subset
- keep the web UI responsive, readable, and predictable
- make the editor save/open flow reliable
- keep the documentation aligned with the shipped behavior

## Next Beta

Target: `v0.1.0-beta.4`

- improve parser coverage for supported Turtle edge cases
- harden blank node, list, and bag handling
- polish validation and conversion error reporting
- improve graph export and preview readability
- add regression tests for parser and API behavior

## Beta Hardening

Target: `v0.1.0-beta.5`

- reduce noisy warnings and CI friction
- keep release packaging stable
- improve the web preview backend configuration story
- tighten the docs for setup, troubleshooting, and release steps
- expand end-to-end checks for CLI, API, and web build paths

## Release Candidate

Target: `v0.1.0-rc.1`

- freeze user-facing features
- fix only blockers, regressions, and data-loss bugs
- verify release packaging and GitHub Pages preview
- complete the final documentation pass
- confirm the CLI, API, and web UI behave consistently

## `v1.0.0`

Target: `v1.0.0`

- publish a stable tag
- treat the supported RDF subset as the documented contract
- keep the CLI, API, and web UI aligned
- ship the beta-tested packaging and preview workflow as stable
- leave room for future RDF/SHACL adapter work in later versions

## After V1

Future versions can focus on larger RDF features without disturbing the stable
`v1` line:

- fuller RDF parser adapters
- complete SHACL integration
- RDF/XML and richer JSON-LD support
- RDF diff and normalization tooling
- longer-running web or workspace features
