# better-tracing

### Community fork 🍴 of `tracing-subscriber` focused on usability and accessibility.

**better-tracing** = **tracing-subscriber** + **smart defaults and features that just work**

Utilities for implementing and composing [`tracing`][tracing] subscribers. This fork provides sensible defaults, accessible formatting, and resolves architectural limitations while maintaining full drop-in compatibility.

| Feature | better-tracing | tracing-subscriber |
|---------|----------------|-------------------|
| **Drop-in compatibility** | ✅ | ✅ |
| **External formatters access exiting span on EXIT/CLOSE** | ✅ | ❌ |
| **Sane defaults with zero configuration** | ⏳ | ❌ |
| **Better builders** you don't have to fight with | ⏳ | ❌ |

See the [CHANGELOG](https://github.com/dra11y/better-tracing/blob/main/CHANGELOG.md) for implemented features and fixes.

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![Documentation (v0.2.x)][docs-v0.2.x-badge]][docs-v0.2.x-url]
[![MIT licensed][mit-badge]][mit-url]
![maintenance status][maint-badge]

[Documentation][docs-url]

[tracing]: https://github.com/tokio-rs/tracing/tree/main/tracing
[crates-badge]: https://img.shields.io/crates/v/better-tracing.svg
[crates-url]: https://crates.io/crates/better-tracing
[docs-badge]: https://docs.rs/better-tracing/badge.svg
[docs-url]: https://docs.rs/better-tracing/latest
[docs-v0.2.x-badge]: https://img.shields.io/badge/docs-v0.2.x-blue
[docs-v0.2.x-url]: https://tracing.rs/better_tracing
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[maint-badge]: https://img.shields.io/badge/maintenance-experimental-blue.svg

*Compiler support: [requires `rustc` 1.65+][msrv]*

[msrv]: #supported-rust-versions

## Supported Rust Versions

This fork follows the same minimum supported Rust version as the upstream `tracing-subscriber`.
The minimum supported version is 1.65. The current version is not guaranteed to build on Rust
versions earlier than the minimum supported version.

better-tracing follows a similar compiler support policy to the upstream project. The current stable Rust compiler and the three most recent minor versions before it will always be supported. For example, if the current stable compiler version is 1.69, the minimum supported version will not be increased past 1.66, three minor versions prior. Increasing the minimum supported compiler version is not considered a semver breaking change as long as doing so complies with this policy.

## License

This project is licensed under the [MIT license](LICENSE).

## Disclaimer

This is an independent community fork of `tracing-subscriber`. This project is not affiliated with, endorsed by, sponsored by, or supported by the Tokio team, the Tokio project, or any of the original `tracing-subscriber` maintainers. All trademarks are the property of their respective owners.

The name "tracing" and related trademarks belong to their respective owners. This fork exists to provide enhanced functionality while maintaining compatibility with the upstream project.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in better-tracing by you, shall be licensed as MIT, without any additional
terms or conditions.
