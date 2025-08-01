//! An [`ExpectedEvent`] defines an event to be matched by the mock
//! subscriber API in the [`subscriber`] module.
//!
//! The expected event should be created with [`expect::event`] and a
//! chain of method calls to describe the assertions we wish to make
//! about the event.
//!
//! # Examples
//!
//! ```
//! use tracing::subscriber::with_default;
//! use better_tracing_mock::{expect, subscriber};
//!
//! let event = expect::event()
//!     .at_level(tracing::Level::INFO)
//!     .with_fields(expect::field("field.name").with_value(&"field_value"));
//!
//! let (subscriber, handle) = subscriber::mock()
//!     .event(event)
//!     .run_with_handle();
//!
//! with_default(subscriber, || {
//!     tracing::info!(field.name = "field_value");
//! });
//!
//! handle.assert_finished();
//! ```
//!
//! [`subscriber`]: mod@crate::subscriber
//! [`expect::event`]: fn@crate::expect::event
use std::fmt;

use crate::{
    ancestry::{ActualAncestry, ExpectedAncestry},
    field,
    metadata::ExpectedMetadata,
    span,
};

/// An expected event.
///
/// For a detailed description and examples, see the documentation for
/// the methods and the [`event`] module.
///
/// [`event`]: mod@crate::event
#[derive(Default, Eq, PartialEq)]
pub struct ExpectedEvent {
    pub(super) fields: Option<field::ExpectedFields>,
    pub(super) ancestry: Option<ExpectedAncestry>,
    pub(super) in_spans: Option<Vec<span::ExpectedSpan>>,
    pub(super) metadata: ExpectedMetadata,
}

impl ExpectedEvent {
    /// Sets a name to expect when matching an event.
    ///
    /// By default, an event's name takes takes the form:
    /// `event <file>:<line>` where `<file>` and `<line>` refer to the
    /// location in the source code where the event was generated.
    ///
    /// To override the name of an event, it has to be constructed
    /// directly, rather than by using the `tracing` crate's macros.
    ///
    /// In general, there are not many use cases for expecting an
    /// event with a particular name, as the value includes the file
    /// name and line number. Assertions about event names are
    /// therefore quite fragile, since they will change as the source
    /// code is modified.
    pub fn named<I>(self, name: I) -> Self
    where
        I: Into<String>,
    {
        Self {
            metadata: ExpectedMetadata {
                name: Some(name.into()),
                ..self.metadata
            },
            ..self
        }
    }

    /// Adds fields to expect when matching an event.
    ///
    /// If an event is recorded with fields that do not match the provided
    /// [`ExpectedFields`], this expectation will fail.
    ///
    /// If the provided field is not present on the recorded event, or
    /// if the value for that field is different, then the expectation
    /// will fail.
    ///
    /// More information on the available validations is available in
    /// the [`ExpectedFields`] documentation.
    ///
    /// # Examples
    ///
    /// ```
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_fields(expect::field("field.name").with_value(&"field_value"));
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     tracing::info!(field.name = "field_value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// A different field value will cause the expectation to fail:
    ///
    /// ```should_panic
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_fields(expect::field("field.name").with_value(&"field_value"));
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     tracing::info!(field.name = "different_field_value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// [`ExpectedFields`]: struct@crate::field::ExpectedFields
    pub fn with_fields<I>(self, fields: I) -> Self
    where
        I: Into<field::ExpectedFields>,
    {
        Self {
            fields: Some(fields.into()),
            ..self
        }
    }

    /// Sets the [`Level`](tracing::Level) to expect when matching an event.
    ///
    /// If an event is recorded at a different level, this expectation
    /// will fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .at_level(tracing::Level::WARN);
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     tracing::warn!("this message is bad news");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// Expecting an event at `INFO` level will fail if the event is
    /// recorded at any other level:
    ///
    /// ```should_panic
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .at_level(tracing::Level::INFO);
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     tracing::warn!("this message is bad news");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    pub fn at_level(self, level: tracing::Level) -> Self {
        Self {
            metadata: ExpectedMetadata {
                level: Some(level),
                ..self.metadata
            },
            ..self
        }
    }

    /// Sets the target to expect when matching events.
    ///
    /// If an event is recorded with a different target, this expectation will fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_target("some_target");
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     tracing::info!(target: "some_target", field = &"value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// The test will fail if the target is different:
    ///
    /// ```should_panic
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_target("some_target");
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     tracing::info!(target: "a_different_target", field = &"value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    pub fn with_target<I>(self, target: I) -> Self
    where
        I: Into<String>,
    {
        Self {
            metadata: ExpectedMetadata {
                target: Some(target.into()),
                ..self.metadata
            },
            ..self
        }
    }

    /// Configures this `ExpectedEvent` to expect the specified
    /// [`ExpectedAncestry`]. An event's ancestry indicates whether is has a
    /// parent or is a root, and whether the parent is explicitly or
    /// contextually assigned.
    ///
    /// An _explicit_ parent span is one passed to the `event!` macro in the
    /// `parent:` field. If no `parent:` field is specified, then the event
    /// will have a contextually determined parent or be a contextual root if
    /// there is no parent.
    ///
    /// If the parent is different from the provided one, this expectation
    /// will fail.
    ///
    /// # Examples
    ///
    /// An explicit or contextual can be matched on an `ExpectedSpan`.
    ///
    /// ```
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let parent = expect::span()
    ///     .named("parent_span")
    ///     .with_target("custom-target")
    ///     .at_level(tracing::Level::INFO);
    /// let event = expect::event()
    ///     .with_ancestry(expect::has_explicit_parent(parent));
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     let parent = tracing::info_span!(target: "custom-target", "parent_span");
    ///     tracing::info!(parent: parent.id(), field = &"value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    /// The functions `expect::has_explicit_parent` and
    /// `expect::has_contextual_parent` take `Into<ExpectedSpan>`, so a string
    /// passed directly will match on a span with that name, or an
    /// [`ExpectedId`] can be passed to match a span with that Id.
    ///
    /// ```
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_ancestry(expect::has_explicit_parent("parent_span"));
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     let parent = tracing::info_span!("parent_span");
    ///     tracing::info!(parent: parent.id(), field = &"value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// In the following example, we expect that the matched event is
    /// an explicit root:
    ///
    /// ```
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_ancestry(expect::is_explicit_root());
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .enter(expect::span())
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     let _guard = tracing::info_span!("contextual parent").entered();
    ///     tracing::info!(parent: None, field = &"value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// When `expect::has_contextual_parent("parent_name")` is passed to
    /// `with_ancestry` then the provided string is the name of the contextual
    /// parent span to expect.
    ///
    /// ```
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_ancestry(expect::has_contextual_parent("parent_span"));
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .enter(expect::span())
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     let parent = tracing::info_span!("parent_span");
    ///     let _guard = parent.enter();
    ///     tracing::info!(field = &"value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// Matching an event recorded outside of a span, a contextual
    /// root:
    ///
    /// ```
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_ancestry(expect::is_contextual_root());
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     tracing::info!(field = &"value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// In the example below, the expectation fails because the event is
    /// recorded with an explicit parent, however a contextual parent is
    /// expected.
    ///
    /// ```should_panic
    /// use tracing::subscriber::with_default;
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_ancestry(expect::has_contextual_parent("parent_span"));
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .enter(expect::span())
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// with_default(subscriber, || {
    ///     let parent = tracing::info_span!("parent_span");
    ///     tracing::info!(parent: parent.id(), field = &"value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// [`ExpectedId`]: struct@crate::span::ExpectedId
    pub fn with_ancestry(self, ancenstry: ExpectedAncestry) -> ExpectedEvent {
        Self {
            ancestry: Some(ancenstry),
            ..self
        }
    }

    /// Validates that the event is emitted within the scope of the
    /// provided `spans`.
    ///
    /// The spans must be provided reverse hierarchy order, so the
    /// closest span to the event would be first, followed by its
    /// parent, and so on.
    ///
    /// If the spans provided do not match the hierarchy of the
    /// recorded event, the expectation will fail.
    ///
    /// **Note**: This validation currently only works with a
    /// [`MockLayer`]. If used with a [`MockSubscriber`], the
    /// expectation will fail directly as it is unimplemented.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_tracing_mock::{expect, layer};
    /// use better_tracing::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
    ///
    /// let event = expect::event().in_scope([
    ///     expect::span().named("parent_span"),
    ///     expect::span().named("grandparent_span")
    /// ]);
    ///
    /// let (layer, handle) = layer::mock()
    ///     .enter(expect::span())
    ///     .enter(expect::span())
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// let _subscriber = better_tracing::registry()
    ///     .with(layer.with_filter(better_tracing::filter::filter_fn(move |_meta| true)))
    ///     .set_default();
    ///
    /// let grandparent = tracing::info_span!("grandparent_span");
    /// let _gp_guard = grandparent.enter();
    /// let parent = tracing::info_span!("parent_span");
    /// let _p_guard = parent.enter();
    /// tracing::info!(field = &"value");
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// The scope must match exactly, otherwise the expectation will fail:
    ///
    /// ```should_panic
    /// use better_tracing_mock::{expect, layer};
    /// use better_tracing::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
    ///
    /// let event = expect::event().in_scope([
    ///     expect::span().named("parent_span"),
    ///     expect::span().named("grandparent_span")
    /// ]);
    ///
    /// let (layer, handle) = layer::mock()
    ///     .enter(expect::span())
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// let _subscriber = better_tracing::registry()
    ///     .with(layer.with_filter(better_tracing::filter::filter_fn(move |_meta| true)))
    ///     .set_default();
    ///
    /// let parent = tracing::info_span!("parent_span");
    /// let _p_guard = parent.enter();
    /// tracing::info!(field = &"value");
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// It is also possible to test that an event has no parent spans
    /// by passing `None` to `in_scope`. If the event is within a
    /// span, the test will fail:
    ///
    /// ```should_panic
    /// use better_tracing_mock::{expect, layer};
    /// use better_tracing::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
    ///
    /// let event = expect::event().in_scope(None);
    ///
    /// let (layer, handle) = layer::mock()
    ///     .enter(expect::span())
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// let _subscriber = better_tracing::registry()
    ///     .with(layer.with_filter(better_tracing::filter::filter_fn(move |_meta| true)))
    ///     .set_default();
    ///
    /// let parent = tracing::info_span!("parent_span");
    /// let _guard = parent.enter();
    /// tracing::info!(field = &"value");
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// [`MockLayer`]: struct@crate::layer::MockLayer
    /// [`MockSubscriber`]: struct@crate::subscriber::MockSubscriber
    #[cfg(feature = "better-tracing")]
    pub fn in_scope(self, spans: impl IntoIterator<Item = span::ExpectedSpan>) -> Self {
        Self {
            in_spans: Some(spans.into_iter().collect()),
            ..self
        }
    }

    /// Provides access to the expected scope (spans) for this expected
    /// event.
    #[cfg(feature = "better-tracing")]
    pub(crate) fn scope_mut(&mut self) -> Option<&mut [span::ExpectedSpan]> {
        self.in_spans.as_mut().map(|s| &mut s[..])
    }

    pub(crate) fn check(
        &mut self,
        event: &tracing::Event<'_>,
        get_ancestry: impl FnOnce() -> ActualAncestry,
        subscriber_name: &str,
    ) {
        let meta = event.metadata();
        let name = meta.name();
        self.metadata
            .check(meta, format_args!("event \"{}\"", name), subscriber_name);
        assert!(
            meta.is_event(),
            "[{}] expected {}, but got {:?}",
            subscriber_name,
            self,
            event
        );
        if let Some(ref mut expected_fields) = self.fields {
            let mut checker = expected_fields.checker(name, subscriber_name);
            event.record(&mut checker);
            checker.finish();
        }

        if let Some(ref expected_ancestry) = self.ancestry {
            let actual_ancestry = get_ancestry();
            expected_ancestry.check(&actual_ancestry, event.metadata().name(), subscriber_name);
        }
    }
}

impl fmt::Display for ExpectedEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "an event{}", self.metadata)
    }
}

impl fmt::Debug for ExpectedEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("MockEvent");

        if let Some(ref name) = self.metadata.name {
            s.field("name", name);
        }

        if let Some(ref target) = self.metadata.target {
            s.field("target", target);
        }

        if let Some(ref level) = self.metadata.level {
            s.field("level", &format_args!("{:?}", level));
        }

        if let Some(ref fields) = self.fields {
            s.field("fields", fields);
        }

        if let Some(ref parent) = self.ancestry {
            s.field("parent", &format_args!("{:?}", parent));
        }

        if let Some(in_spans) = &self.in_spans {
            s.field("in_spans", in_spans);
        }

        s.finish()
    }
}
