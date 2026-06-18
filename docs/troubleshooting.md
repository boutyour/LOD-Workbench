# Troubleshooting

This page collects the most common setup and parsing issues.

## API Connectivity

- If the web UI says the API is unreachable, verify that `lod-api` is running
  on `127.0.0.1:8080`.
- If you changed the API base URL, update `VITE_API_URL` before starting the
  frontend.
- If you use a remote preview build, set the repository variable
  `VITE_API_URL` to a reachable backend URL before running the frontend.

## RDF Parsing

- If parsing fails, check that the example uses the supported Turtle/N-Triples
  subset.
- Make sure prefixes end with a full IRI and a terminating `.`.
- Confirm that quoted literals are closed and that blank node blocks are
  balanced.
- If you use RDF collections or bags, keep the nested punctuation intact.

## Validation

- Validation warns about non-HTTP IRIs by design.
- SHACL support is currently represented as a lightweight notice in V1.
- If a sample fails validation, inspect the reported line number and correct
  the offending triple.

## Graph Export

- If an exported SVG looks clipped, use the built-in export button from the
  visualizer.
- The graph viewport can be zoomed and panned with the mouse wheel and drag.

## General Tips

- Run `cargo test --workspace` after parser or service changes.
- Run `npm run build` in `apps/web` after frontend changes.
- If a workflow behaves unexpectedly, inspect the matching GitHub Actions log
  for the CI or Pages job.
