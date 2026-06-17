# Release

<!-- markdownlint-disable MD013 -->

LOD Workbench uses branches, tags, GitHub Actions, and GitHub Pages to publish
beta builds and release snapshots.

## Branches

- `main` tracks the stable development line
- `beta` tracks the preview branch used for Pages deployment

## Beta Deploy

- push to `beta` to trigger the beta Pages workflow
- set `VITE_API_URL` to a reachable API base URL
- the Pages build uses the React/Vite client from `apps/web`

## GitHub Releases

- push a tag such as `v0.1.0-beta.2` to trigger the release workflow
- beta and release-candidate tags are treated as prereleases
- the release workflow reuses the shared build job from
  `.github/workflows/reusable-build.yml`
- release assets include Linux binaries and the web client archive

## Typical Release Flow

1. Merge the target work into `beta`.
2. Verify the beta Pages deployment.
3. Tag the beta commit.
4. Push the tag to GitHub.
5. Check the GitHub Release page for the generated prerelease assets.

## Versioning Notes

- semantic version tags are used for published snapshots
- beta tags are intended for preview and testing
- stable tags can be used once the release is ready for wider distribution

## Related Workflows

- `CI` uses the reusable build workflow for linting, build, and test checks
- `Beta Pages` uses the reusable build workflow and then deploys the web client
- `Docs` validates Markdown links and structure before documentation changes
  land

<!-- markdownlint-enable MD013 -->
