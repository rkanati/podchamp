# Release procedure
- `cargo update`
- test
    - fix problems from dep update if necessary
- bump version number
    - follow semver; does this release contain a breaking change?
- update `CHANGELOG.md`
    - chicken-and-egg with `git tag`; figure out the release link url
- `git tag`
- release on github?
- `cargo publish`

