# Release

<!-- markdownlint-disable MD013 -->

LOD Workbench keeps release handling simple: one CI workflow validates the
codebase, and tags can be used when you want to cut a manual snapshot.

## Branches

- `main` tracks the stable development line
- `beta` can be used for preview testing when needed

## Manual Snapshot Flow

1. Merge the target work into `beta` or `main`.
2. Run the CI workflow locally or wait for GitHub Actions to pass.
3. Tag the commit when you want a snapshot.
4. Publish the tag from your local git workflow if you need an archive.

## Versioning Notes

- semantic version tags can still be used for published snapshots
- beta tags are optional and only useful for preview/testing
- stable tags can be used once the release is ready for wider distribution

## Related Workflows

- `CI` runs the Rust checks and web build in one place

<!-- markdownlint-enable MD013 -->
