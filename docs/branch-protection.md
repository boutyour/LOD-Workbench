# Branch Protection

These are the recommended GitHub settings for the two long-lived branches.

## `main`

- require pull requests before merging
- require at least one review
- require status checks to pass before merging
- include the CI check
- disallow force pushes
- disallow branch deletion

## `beta`

- require pull requests for changes that should reach the preview site
- require the CI check to pass
- disallow force pushes
- keep branch deletion disabled

## Why This Helps

- `main` stays stable and release-ready
- `beta` stays useful for preview testing
- the single CI workflow keeps the checks consistent across branches

## Suggested Checks

- `CI / Rust checks`
- `CI / Web build`
- `CI / Docs checks`

## Note

GitHub branch protection is configured in the repository settings UI rather than
in code, so this page is the operational guide for setting it up.
