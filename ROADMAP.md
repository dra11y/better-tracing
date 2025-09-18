# better-tracing Roadmap

## Vision

A drop-in fork of `tracing-subscriber` that's actually easy to use. No more 500+ lines of boilerplate!

**Core Goal**: `better_tracing::fmt().init()` should "just work."

## Priority Fixes

### Span Context Bug

**Problem**: Custom formatters can only see parent span during EXIT events.

```rust
// Current behavior
Built-in formatter: parent_function:child_operation ✅
Custom formatter:   parent_function ❌ (missing child)
```

**Solution**: Extend `FormatEvent` trait to provide full span hierarchy to custom formatters.

### External API Parity

Built-in formatters have privileged access to internal mechanisms. External formatters are limited to reduced public APIs.

**Fix**: Make all internal span info public. Custom formatters should have same capabilities as built-in ones.

### Accessibility & Usability

- Sensible defaults that work out-of-the-box
- Allow full customization of output format including:
  - fully customize file/path/line display
  - override field names and values
  - override overall span display format
  - granular control over field and span visibility
  - customize log format for screen readers, Braille displays, and other external systems
- Flexible builder pattern that doesn't break when changing options

## Implementation

**Span Context Architecture**: Fix `FmtContext::lookup_current()` method to return exiting / closing span context.

**API Parity**: Audit `pub(crate)`/internal items, create public accessor functions, ensure external formatters have full span information access.
