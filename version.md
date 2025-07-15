# Version

To update the version:

- Add the new version to [CHANGELOG.md](CHANGELOG.md), and note key changes.
- Update the version in [Makefile](/Makefile).
- Update the version in [sddr-gen/Cargo.toml](/rust/sdrr-gen/Cargo.toml).
- Update the version in [sddr-info/Cargo.toml](/rust/sdrr-info/Cargo.toml).
- Update the version in [sdrr-common/Cargo.toml](/rust/sdrr-common/Cargo.toml).
- Update the version consts `MAX_VERSION_*` in [rust/sdrr-fw-parser/src/lib.rs](/rust/sdrr-fw-parser/src/lib.rs).
