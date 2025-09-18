# better-tracing

### Community fork 🍴 of `tracing-subscriber` focused on usability and accessibility.

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![changelog][changelog-badge]][changelog-url]
![maintenance status][maint-badge]

**better-tracing** = **tracing-subscriber** + **smart defaults and features that just work**

See the [CHANGELOG](https://github.com/dra11y/better-tracing/blob/main/CHANGELOG.md) for implemented features and fixes.

Utilities for implementing and composing [`tracing`][tracing] subscribers. This fork provides sensible defaults, accessible formatting, and resolves architectural limitations while maintaining drop-in compatibility.

**Backward compatibility and compatibility with 3rd party crates that might use `tracing_subscriber` will be maintained on a best-effort basis.**

| Feature | better-tracing | tracing-subscriber |
|---------|----------------|-------------------|
| **Drop-in compatibility** | ✅ | ✅ |
| **Easier time formats** | ✅ | ❌ |
| **External formatters access exiting span on EXIT/CLOSE** | ✅ | ❌ |
| **Sane defaults with zero configuration** | ⏳ | ❌ |
| **Better builders** you don't have to fight with | ⏳ | ❌ |

*Compiler support: [requires `rustc` 1.65+][msrv]*

[msrv]: #supported-rust-versions

## Fork Improvements

### Easier Time Formats

Configure timestamps without extra crates and keep using `.with_timer(...)`:

- RFC 3339 (UTC):
  - `SystemTime::rfc3339_seconds()`     → `YYYY-MM-DDTHH:MM:SSZ`
  - `SystemTime::rfc3339_millis()`      → `YYYY-MM-DDTHH:MM:SS.mmmZ`
  - `SystemTime::rfc3339_nanos()`       → `YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ`
- Unix epoch (UTC):
  - `SystemTime::unix_seconds()` / `unix_millis()` / `unix_micros()` / `unix_nanos()`
- Time-only (no date, great for dev logs):
  - `SystemTime::time_only_secs()`      → `HH:MM:SS`
  - `SystemTime::time_only_millis()`    → `HH:MM:SS.mmm`
  - `SystemTime::time_only_micros()`    → `HH:MM:SS.uuuuuu`

Examples:
```rust
use tracing_subscriber::fmt::time::SystemTime;
let subscriber = tracing_subscriber::fmt()
    .with_timer(SystemTime::time_only_millis())
    .finish();
```
```rust
use tracing_subscriber::fmt::time::SystemTime;
let subscriber = tracing_subscriber::fmt()
    .with_timer(SystemTime::rfc3339_millis())
    .finish();
```

With optional engines, pick the matching helpers: `fmt::time::ChronoUtc::*` (chrono), `fmt::time::UtcTime::*` (time), or `fmt::time::SystemTime::*` (default).

### Fixed Span Context for EXIT/CLOSE Events

In `tracing-subscriber`, `lookup_current()` returned the parent span instead of the exiting span during `FmtSpan::EXIT` and `FmtSpan::CLOSE` events. `better-tracing` fixes this.

```rust
use tracing_subscriber::fmt::format::FmtSpan;

// Custom formatter that can now access the exiting span
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};
use tracing::{Event, Subscriber};
use std::fmt;

struct MyFormatter;

impl<S, N> FormatEvent<S, N> for MyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Now returns the correct exiting span during EXIT/CLOSE events
        if let Some(span) = ctx.lookup_current() {
            write!(writer, "{}:", span.name())?;
        }
        Ok(())
    }
}

// Set up subscriber with the custom formatter and EXIT events
let subscriber = tracing_subscriber::fmt()
    .with_span_events(FmtSpan::EXIT)
    .event_format(MyFormatter)
    .finish();
```

## Adapted from upstream `tracing-subscriber`

[`tracing`] is a framework for instrumenting Rust programs to collect
scoped, structured, and async-aware diagnostics. The [`Subscriber`] trait
represents the functionality necessary to collect this trace data. This
crate contains tools for composing subscribers out of smaller units of
behaviour, and batteries-included implementations of common subscriber
functionality.

`better-tracing` is intended for use by both `Subscriber` authors and
application authors using `tracing` to instrument their applications.

## `Layer`s and `Filter`s

The most important component of the `better-tracing` API is the
[`Layer`] trait, which provides a composable abstraction for building
[`Subscriber`]s. Like the [`Subscriber`] trait, a [`Layer`] defines a
particular behavior for collecting trace data. Unlike [`Subscriber`]s,
which implement a *complete* strategy for how trace data is collected,
[`Layer`]s provide *modular* implementations of specific behaviors.
Therefore, they can be [composed together] to form a [`Subscriber`] which is
capable of recording traces in a variety of ways. See the [`layer` module's
documentation][layer] for details on using [`Layer`]s.

In addition, the [`Filter`] trait defines an interface for filtering what
spans and events are recorded by a particular layer. This allows different
[`Layer`]s to handle separate subsets of the trace data emitted by a
program. See the [documentation on per-layer filtering][plf] for more
information on using [`Filter`]s.

[`Layer`]: crate::layer::Layer
[composed together]: crate::layer#composing-layers
[layer]: crate::layer
[`Filter`]: crate::layer::Filter
[plf]: crate::layer#per-layer-filtering

## Included Subscribers

The following `Subscriber`s are provided for application authors:

- [`fmt`] - Formats and logs tracing data (requires the `fmt` feature flag)

## Feature Flags

- `std`: Enables APIs that depend on the Rust standard library
  (enabled by default).
- `alloc`: Depend on [`liballoc`] (enabled by "std").
- `env-filter`: Enables the [`EnvFilter`] type, which implements filtering
  similar to the [`env_logger` crate]. **Requires "std"**.
- `fmt`: Enables the [`fmt`] module, which provides a subscriber
  implementation for printing formatted representations of trace events.
  Enabled by default. **Requires "registry" and "std"**.
- `ansi`: Enables `fmt` support for ANSI terminal colors. Enabled by
  default.
- `registry`: enables the [`registry`] module. Enabled by default.
  **Requires "std"**.
- `json`: Enables `fmt` support for JSON output. In JSON output, the ANSI
  feature does nothing. **Requires "fmt" and "std"**.
- `local-time`: Enables local time formatting when using the [`time`
  crate]'s timestamp formatters with the `fmt` subscriber.

[`registry`]: mod@registry

### Optional Dependencies

- [`tracing-log`]: Enables better formatting for events emitted by `log`
  macros in the `fmt` subscriber. Enabled by default.
- [`time`][`time` crate]: Enables support for using the [`time` crate] for timestamp
  formatting in the `fmt` subscriber.
- [`smallvec`]: Causes the `EnvFilter` type to use the `smallvec` crate (rather
  than `Vec`) as a performance optimization. Enabled by default.
- [`parking_lot`]: Use the `parking_lot` crate's `RwLock` implementation
  rather than the Rust standard library's implementation.

### `no_std` Support

In embedded systems and other bare-metal applications, `tracing` can be
used without requiring the Rust standard library, although some features are
disabled. Although most of the APIs provided by `better-tracing`, such
as [`fmt`] and [`EnvFilter`], require the standard library, some
functionality, such as the [`Layer`] trait, can still be used in
`no_std` environments.

The dependency on the standard library is controlled by two crate feature
flags, "std", which enables the dependency on [`libstd`], and "alloc", which
enables the dependency on [`liballoc`] (and is enabled by the "std"
feature). These features are enabled by default, but `no_std` users can
disable them using:

```toml
# Cargo.toml
better-tracing = { version = "0.3", default-features = false }
```

Additional APIs are available when [`liballoc`] is available. To enable
`liballoc` but not `std`, use:

```toml
# Cargo.toml
better-tracing = { version = "0.3", default-features = false, features = ["alloc"] }
```

### Unstable Features

These feature flags enable **unstable** features. The public API may break in 0.1.x
releases. To enable these features, the `--cfg tracing_unstable` must be passed to
`rustc` when compiling.

The following unstable feature flags are currently available:

* `valuable`: Enables support for serializing values recorded using the
  [`valuable`] crate as structured JSON in the [`format::Json`] formatter.

### Enabling Unstable Features

The easiest way to set the `tracing_unstable` cfg is to use the `RUSTFLAGS`
env variable when running `cargo` commands:

```shell
RUSTFLAGS="--cfg tracing_unstable" cargo build
```
Alternatively, the following can be added to the `.cargo/config` file in a
project to automatically enable the cfg flag for that project:

```toml
[build]
rustflags = ["--cfg", "tracing_unstable"]
```

[feature flags]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
[`valuable`]: https://crates.io/crates/valuable
[`format::Json`]: crate::fmt::format::Json

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

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in better-tracing by you, shall be licensed as MIT, without any additional
terms or conditions.

[tracing]: https://github.com/tokio-rs/tracing/tree/main/tracing
[tracing-subscriber]: https://github.com/tokio-rs/tracing/tree/main/tracing-subscriber
[crates-badge]: https://img.shields.io/crates/v/better-tracing.svg
[crates-url]: https://crates.io/crates/better-tracing
[docs-badge]: https://docs.rs/better-tracing/badge.svg
[docs-url]: https://docs.rs/better-tracing/latest
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[changelog-badge]: https://img.shields.io/badge/changelog-Keep%20a%20Changelog-E05735?labelColor=555555&logo=keepachangelog
[maint-badge]: https://img.shields.io/badge/maintenance-experimental-blue.svg
[changelog-url]: CHANGELOG.md
[`Subscriber`]: tracing_core::subscriber::Subscriber
[`tracing`]: https://docs.rs/tracing/latest/tracing
[`EnvFilter`]: filter::EnvFilter
[`fmt`]: mod@fmt
[`tracing`]: https://crates.io/crates/tracing
[`tracing-log`]: https://crates.io/crates/tracing-log
[`smallvec`]: https://crates.io/crates/smallvec
[`env_logger` crate]: https://crates.io/crates/env_logger
[`parking_lot`]: https://crates.io/crates/parking_lot
[`time` crate]: https://crates.io/crates/time
[`liballoc`]: https://doc.rust-lang.org/alloc/index.html
[`libstd`]: https://doc.rust-lang.org/std/index.html

License: MIT
