# Branch Protection

These are the recommended GitHub settings for the two long-lived branches.

## `main`

- require pull requests before merging
- require at least one review
- require status checks to pass before merging
- include the CI, Docs, and Release-related checks
- disallow force pushes
- disallow branch deletion

## `beta`

- require pull requests for changes that should reach the preview site
- require the CI, Docs, and Beta Pages checks to pass
- disallow force pushes
- keep branch deletion disabled

## Why This Helps

- `main` stays stable and release-ready
- `beta` stays deployable for preview testing
- the shared build workflow makes the checks consistent across branches

## Suggested Checks

- `CI / build`
- `Docs / markdown`
- `Beta Pages / build`
- `Release / build`

## Note

GitHub branch protection is configured in the repository settings UI rather than
in code, so this page is the operational guide for setting it up.
