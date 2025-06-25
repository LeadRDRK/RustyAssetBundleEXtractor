# runirip [![Build Status]][actions] [![Latest Version]][crates.io] [![Docs]][docs.rs] [![License_MIT]][license_mit] [![License_APACHE]][license_apache] 

[Build Status]: https://img.shields.io/github/actions/workflow/status/LeadRDRK/runirip/ci.yml?branch=main
[actions]: https://github.com/LeadRDRK/runirip/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/runirip
[crates.io]: https://crates.io/crates/runirip
[Docs]: https://docs.rs/runirip/badge.svg
[docs.rs]: https://docs.rs/crate/runirip/
[License_MIT]: https://img.shields.io/badge/License-MIT-yellow.svg
[license_mit]: https://raw.githubusercontent.com/LeadRDRK/runirip/main/LICENSE-MIT
[License_APACHE]: https://img.shields.io/badge/License-Apache%202.0-blue.svg
[license_apache]: https://raw.githubusercontent.com/LeadRDRK/runirip/main/LICENSE-APACHE


runirip is a Rust library that allows you to manipulate various Unity asset file formats. It is a fork of [rabex](https://github.com/UniversalGameExtraction/RustyAssetBundleEXtractor) that aims to be usable in production.

## Feature flags

All of these features are enabled by default.
- `unitycn_encryption`: Enables support for decrypting encrypted UnityCN assets.
- `objects`: Enables the [`objects`](https://crates.io/crates/runirip-objects) crate which contains struct definitions for Unity classes to be parsed as. Depends on `serde`.
- `serde`: Enables `serde` serialization/deserialization support.
- `lzma`, `lz4`, `brotli`: Enables support for the corresponding compression method.

## Examples

See [`/examples`](/examples).

## Notes

### TODO

- Parsers:

  - [x] SerializedFile
  - [x] BundleFile
  - [ ] WebFile

- Object Classes:

  - [x] Generator
  - [x] Parser
  - [ ] Writer
  - [ ] Export Functions

- Tests:

  - [ ] Normal Tests
  - [ ] Artificing Test Files
  - [ ] 100% Coverage

- Other:
  - [x] Feature config

## License
runirip is dual-licensed under Apache 2.0 and MIT. You can choose between one of them if you use this library.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for more details.
