# Release procedure
- bump version number
    - follow semver; does this release contain a breaking change?
- `cargo update`
- test
    - fix problems from dep update if necessary
- `git commit` as usual
- update `CHANGELOG.md`
    - chicken-and-egg with `git tag`; figure out the release link url
- `git commit` with shortlog for release
- `git tag -s $VERSION`
- `git push`
- `git push --tags`
- release on github?
    - probably not; tags are fine by themselves
- `cargo publish`

