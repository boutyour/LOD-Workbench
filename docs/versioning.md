# Versioning

LOD Workbench uses branches for active development and tags for published
versions.

## Branches

- `main` is the stable development line
- `beta` is the preview line used for beta testing and deployment
- feature branches are used for isolated work before merging

## Tags

- `v0.1.0-beta.x` marks beta snapshots
- `v0.1.0-rc.x` marks release candidates
- `v1.0.0` marks the first stable release

## Recommended Flow

1. Develop work on a feature branch.
2. Merge the feature into `beta` for preview testing.
3. Tag the tested `beta` commit as a beta release.
4. When behavior is frozen, tag a release candidate.
5. When the project is ready, tag `v1.0.0`.

## Practical Rules

- never retag a release that has already been pushed publicly
- create a new tag for each beta or release candidate
- use `beta` for preview validation and `main` for the stable line
- keep tags aligned with the exact commit you want to publish
