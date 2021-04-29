# Release procedure
- bump version number
    - follow semver; does this release contain a breaking change?
- `cargo update`
- test
    - fix problems from dep update if necessary
- update `CHANGELOG.md`
    - chicken-and-egg with `git tag`; figure out the release link url
- `git tag`
- release on github?
    - probably not; tags are fine by themselves
- `cargo publish`

