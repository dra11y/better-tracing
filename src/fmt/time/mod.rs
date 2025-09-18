//! Formatters for event timestamps.
use crate::fmt::format::Writer;
use std::fmt;
use std::time as stdtime;
use std::time::Instant;

mod datetime;

#[cfg(feature = "time")]
mod time_crate;

#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub use time_crate::UtcTime;

#[cfg(feature = "local-time")]
#[cfg_attr(docsrs, doc(cfg(all(unsound_local_offset, feature = "local-time"))))]
pub use time_crate::LocalTime;

#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub use time_crate::OffsetTime;

/// [`chrono`]-based implementation for [`FormatTime`].
#[cfg(feature = "chrono")]
mod chrono_crate;

#[cfg(feature = "chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
pub use chrono_crate::ChronoLocal;

#[cfg(feature = "chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
pub use chrono_crate::ChronoUtc;

/// A type that can measure and format the current time.
///
/// This trait is used by `Format` to include a timestamp with each `Event` when it is logged.
///
/// Notable default implementations of this trait are `SystemTime` and `()`. The former prints the
/// current time as reported by `std::time::SystemTime`, and the latter does not print the current
/// time at all. `FormatTime` is also automatically implemented for any function pointer with the
/// appropriate signature.
///
/// The full list of provided implementations can be found in [`time`].
///
/// [`time`]: self
pub trait FormatTime {
    /// Measure and write out the current time.
    ///
    /// When `format_time` is called, implementors should get the current time using their desired
    /// mechanism, and write it out to the given `fmt::Write`. Implementors must insert a trailing
    /// space themselves if they wish to separate the time from subsequent log message text.
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result;
}

// --- Core time architecture: Clock + Formatter + Timer -----------------------

/// Captures the notion of "now" and returns a snapshot value.
/// Captures the notion of time ("now") and returns a snapshot value.
pub trait Clock {
    /// The concrete timestamp representation captured by this clock.
    type Snapshot;
    /// Get the current time snapshot.
    fn now(&self) -> Self::Snapshot;
}

/// Formats a captured snapshot into the Writer without allocating.
/// Formats a captured time snapshot into the writer with no allocations.
pub trait TimestampFormatter<Input> {
    /// Write a textual representation of `input` into `w`.
    fn format(&self, input: &Input, w: &mut Writer<'_>) -> fmt::Result;
}

/// A combinator that implements `FormatTime` as `F(C::now())`.
/// Composes a `Clock` and a `TimestampFormatter` to implement `FormatTime`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Timer<C, F>(pub C, pub F);

impl<C, F> FormatTime for Timer<C, F>
where
    C: Clock,
    F: TimestampFormatter<C::Snapshot>,
{
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        let snap = self.0.now();
        self.1.format(&snap, w)
    }
}

/// System wall-clock now.
/// A `Clock` that returns `std::time::SystemTime::now()`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    type Snapshot = stdtime::SystemTime;
    fn now(&self) -> Self::Snapshot {
        stdtime::SystemTime::now()
    }
}

/// RFC3339 formatter with configurable fractional digits and optional 'Z'.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rfc3339<const DIGITS: u8, const Z: bool>;

impl<const D: u8, const Z: bool> TimestampFormatter<stdtime::SystemTime> for Rfc3339<D, Z> {
    fn format(&self, input: &stdtime::SystemTime, w: &mut Writer<'_>) -> fmt::Result {
        // Leverage the existing no-deps DateTime to render RFC3339 with truncation.
        let dt = datetime::DateTime::from(*input);
        let digits = if D > 9 { 9 } else { D } as u8;
        dt.fmt_rfc3339_with_subsec_to(w, digits, Z)
    }
}

/// Returns a new `SystemTime` timestamp provider.
///
/// This can then be configured further to determine how timestamps should be
/// configured.
///
/// This is equivalent to calling
/// ```rust
/// # fn timer() -> tracing_subscriber::fmt::time::SystemTime {
/// tracing_subscriber::fmt::time::SystemTime::default()
/// # }
/// ```
pub fn time() -> SystemTime {
    SystemTime
}

/// Returns a new `Uptime` timestamp provider.
///
/// With this timer, timestamps will be formatted with the amount of time
/// elapsed since the timestamp provider was constructed.
///
/// This can then be configured further to determine how timestamps should be
/// configured.
///
/// This is equivalent to calling
/// ```rust
/// # fn timer() -> tracing_subscriber::fmt::time::Uptime {
/// tracing_subscriber::fmt::time::Uptime::default()
/// # }
/// ```
pub fn uptime() -> Uptime {
    Uptime::default()
}

impl<F> FormatTime for &F
where
    F: FormatTime,
{
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        (*self).format_time(w)
    }
}

impl FormatTime for () {
    fn format_time(&self, _: &mut Writer<'_>) -> fmt::Result {
        Ok(())
    }
}

impl FormatTime for fn(&mut Writer<'_>) -> fmt::Result {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        (*self)(w)
    }
}

/// Retrieve and print the current wall-clock time.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct SystemTime;

/// Retrieve and print the relative elapsed wall-clock time since an epoch.
///
/// The `Default` implementation for `Uptime` makes the epoch the current time.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Uptime {
    epoch: Instant,
}

impl Default for Uptime {
    fn default() -> Self {
        Uptime {
            epoch: Instant::now(),
        }
    }
}

impl From<Instant> for Uptime {
    fn from(epoch: Instant) -> Self {
        Uptime { epoch }
    }
}

impl FormatTime for SystemTime {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        // Delegate to the unified path: SystemClock + RFC3339 micros with Z.
        Timer(SystemClock, Rfc3339::<6, true>).format_time(w)
    }
}

impl FormatTime for Uptime {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        let e = self.epoch.elapsed();
        write!(w, "{:4}.{:09}s", e.as_secs(), e.subsec_nanos())
    }
}
