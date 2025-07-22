# Version

## Updating Version Number

To update the version:

- Add the new version to [CHANGELOG.md](CHANGELOG.md), and note key changes.
- Update the version in [Makefile](/Makefile).
- Update the version in [sddr-gen/Cargo.toml](/rust/sdrr-gen/Cargo.toml).
- Update the version in [sddr-info/Cargo.toml](/rust/sdrr-info/Cargo.toml).
- Update the version in [sdrr-common/Cargo.toml](/rust/sdrr-common/Cargo.toml).
- Update the version consts `MAX_VERSION_*` in [rust/sdrr-fw-parser/src/lib.rs](/rust/sdrr-fw-parser/src/lib.rs).

## Release Process

Ensure all changes are committed, including the [version number updates](#updating-version-number).

```bash
git pull
git push
```

Locally run the following tests:

```bash
ci/build.sh test
ci/build.sh ci
ci/build.sh release v<x.y.z>
```

Tag the version in git:

```bash
git tag -s -a v<x.y.z> -m "Release v<x.y.z>
git push origin v<x.y.z>
```
