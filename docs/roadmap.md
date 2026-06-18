# Roadmap

This roadmap covers the path from the current beta line to `v1.0.0`.

## Beta

- keep the single CI workflow green
- keep the GitHub Pages preview working
- stabilize the parser on the supported RDF subset
- keep the web UI responsive and predictable

## Beta.1

- improve parser coverage for supported Turtle edge cases
- polish validation and conversion error reporting
- keep graph visualization readable for larger RDF samples
- tighten documentation around setup and usage

## Beta.2

- harden the API and web preview flows
- improve import and export handling for the editor
- reduce noisy warnings and CI friction
- expand regression tests for parser and service behavior

## Release Candidate

- freeze the user-facing behavior
- fix only blockers, regressions, and data-loss bugs
- verify release packaging and GitHub Pages preview
- complete release notes and final documentation pass

## V1.0.0

- publish a stable tag
- treat the supported RDF subset as the documented contract
- keep the CLI, API, and web UI aligned
- leave room for future RDF/SHACL adapter work in later versions
