//! Define expectations to validate fields on events and spans.
//!
//! The [`ExpectedField`] struct define expected values for fields in
//! order to match events and spans via the mock subscriber API in the
//! [`subscriber`] module.
//!
//! Expected fields should be created with [`expect::field`] and a
//! chain of method calls to specify the field value and additional
//! fields as necessary.
//!
//! # Examples
//!
//! The simplest case is to expect that an event has a field with a
//! specific name, without any expectation about the value:
//!
//! ```
//! use better_tracing_mock::{expect, subscriber};
//!
//! let event = expect::event()
//!     .with_fields(expect::field("field_name"));
//!
//! let (subscriber, handle) = subscriber::mock()
//!     .event(event)
//!     .run_with_handle();
//!
//! tracing::subscriber::with_default(subscriber, || {
//!     tracing::info!(field_name = "value");
//! });
//!
//! handle.assert_finished();
//! ```
//!
//! It is possible to expect multiple fields and specify the value for
//! each of them:
//!
//! ```
//! use better_tracing_mock::{expect, subscriber};
//!
//! let event = expect::event().with_fields(
//!     expect::field("string_field")
//!         .with_value(&"field_value")
//!         .and(expect::field("integer_field").with_value(&54_i64))
//!         .and(expect::field("bool_field").with_value(&true)),
//! );
//!
//! let (subscriber, handle) = subscriber::mock()
//!     .event(event)
//!     .run_with_handle();
//!
//! tracing::subscriber::with_default(subscriber, || {
//!     tracing::info!(
//!         string_field = "field_value",
//!         integer_field = 54_i64,
//!         bool_field = true,
//!     );
//! });
//!
//! handle.assert_finished();
//! ```
//!
//! If an expected field is not present, or if the value of the field
//! is different, the test will fail. In this example, the value is
//! different:
//!
//! ```should_panic
//! use better_tracing_mock::{expect, subscriber};
//!
//! let event = expect::event()
//!     .with_fields(expect::field("field_name").with_value(&"value"));
//!
//! let (subscriber, handle) = subscriber::mock()
//!     .event(event)
//!     .run_with_handle();
//!
//! tracing::subscriber::with_default(subscriber, || {
//!     tracing::info!(field_name = "different value");
//! });
//!
//! handle.assert_finished();
//! ```
//!
//! [`subscriber`]: mod@crate::subscriber
//! [`expect::field`]: fn@crate::expect::field
use std::{collections::HashMap, fmt};

use tracing::{
    callsite,
    callsite::Callsite,
    field::{self, Field, Value, Visit},
    metadata::Kind,
};

/// An expectation for multiple fields.
///
/// For a detailed description and examples, see the documentation for
/// the methods and the [`field`] module.
///
/// [`field`]: mod@crate::field
#[derive(Default, Debug, Eq, PartialEq)]
pub struct ExpectedFields {
    fields: HashMap<String, ExpectedValue>,
    only: bool,
}

/// An expected field.
///
/// For a detailed description and examples, see the documentation for
/// the methods and the [`field`] module.
///
/// [`field`]: mod@crate::field
#[derive(Debug)]
pub struct ExpectedField {
    pub(super) name: String,
    pub(super) value: ExpectedValue,
}

#[derive(Debug)]
pub(crate) enum ExpectedValue {
    F64(f64),
    I64(i64),
    U64(u64),
    Bool(bool),
    Str(String),
    Debug(String),
    Any,
}

impl Eq for ExpectedValue {}

impl PartialEq for ExpectedValue {
    fn eq(&self, other: &Self) -> bool {
        use ExpectedValue::*;

        match (self, other) {
            (F64(a), F64(b)) => {
                debug_assert!(!a.is_nan());
                debug_assert!(!b.is_nan());

                a.eq(b)
            }
            (I64(a), I64(b)) => a.eq(b),
            (U64(a), U64(b)) => a.eq(b),
            (Bool(a), Bool(b)) => a.eq(b),
            (Str(a), Str(b)) => a.eq(b),
            (Debug(a), Debug(b)) => a.eq(b),
            (Any, _) => true,
            (_, Any) => true,
            _ => false,
        }
    }
}

impl ExpectedField {
    /// Sets the value to expect when matching this field.
    ///
    /// If the recorded value for this field is different, the
    /// expectation will fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_fields(expect::field("field_name").with_value(&"value"));
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// tracing::subscriber::with_default(subscriber, || {
    ///     tracing::info!(field_name = "value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// A different value will cause the test to fail:
    ///
    /// ```should_panic
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_fields(expect::field("field_name").with_value(&"value"));
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// tracing::subscriber::with_default(subscriber, || {
    ///     tracing::info!(field_name = "different value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    pub fn with_value(self, value: &dyn Value) -> Self {
        Self {
            value: ExpectedValue::from(value),
            ..self
        }
    }

    /// Adds an additional [`ExpectedField`] to be matched.
    ///
    /// Any fields introduced by `.and` must also match. If any fields
    /// are not present, or if the value for any field is different,
    /// then the expectation will fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event().with_fields(
    ///     expect::field("field")
    ///         .with_value(&"value")
    ///         .and(expect::field("another_field").with_value(&42)),
    /// );
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// tracing::subscriber::with_default(subscriber, || {
    ///     tracing::info!(
    ///         field = "value",
    ///         another_field = 42,
    ///     );
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// If the second field is not present, the test will fail:
    ///
    /// ```should_panic
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event().with_fields(
    ///     expect::field("field")
    ///         .with_value(&"value")
    ///         .and(expect::field("another_field").with_value(&42)),
    /// );
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// tracing::subscriber::with_default(subscriber, || {
    ///     tracing::info!(field = "value");
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    pub fn and(self, other: ExpectedField) -> ExpectedFields {
        ExpectedFields {
            fields: HashMap::new(),
            only: false,
        }
        .and(self)
        .and(other)
    }

    /// Indicates that no fields other than those specified should be
    /// expected.
    ///
    /// If additional fields are present on the recorded event or span,
    /// the expectation will fail.
    ///
    /// # Examples
    ///
    /// The following test passes despite the recorded event having
    /// fields that were not expected because `only` was not
    /// used:
    ///
    /// ```
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_fields(expect::field("field").with_value(&"value"));
    ///
    /// let (subscriber, handle) = subscriber::mock().event(event).run_with_handle();
    ///
    /// tracing::subscriber::with_default(subscriber, || {
    ///     tracing::info!(field = "value", another_field = 42,);
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// If we include `only` on the `ExpectedField` then the test
    /// will fail:
    ///
    /// ```should_panic
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event()
    ///     .with_fields(expect::field("field").with_value(&"value").only());
    ///
    /// let (subscriber, handle) = subscriber::mock().event(event).run_with_handle();
    ///
    /// tracing::subscriber::with_default(subscriber, || {
    ///     tracing::info!(field = "value", another_field = 42,);
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    pub fn only(self) -> ExpectedFields {
        ExpectedFields {
            fields: HashMap::new(),
            only: true,
        }
        .and(self)
    }
}

impl From<ExpectedField> for ExpectedFields {
    fn from(field: ExpectedField) -> Self {
        ExpectedFields {
            fields: HashMap::new(),
            only: false,
        }
        .and(field)
    }
}

impl ExpectedFields {
    /// Adds an additional [`ExpectedField`] to be matched.
    ///
    /// All fields must match, if any of them are not present, or if
    /// the value for any field is different, the expectation will
    /// fail.
    ///
    /// This method performs the same function as
    /// [`ExpectedField::and`], but applies in the case where there are
    /// already multiple fields expected.
    ///
    /// # Examples
    ///
    /// ```
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event().with_fields(
    ///     expect::field("field")
    ///         .with_value(&"value")
    ///         .and(expect::field("another_field").with_value(&42))
    ///         .and(expect::field("a_third_field").with_value(&true)),
    /// );
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// tracing::subscriber::with_default(subscriber, || {
    ///     tracing::info!(
    ///         field = "value",
    ///         another_field = 42,
    ///         a_third_field = true,
    ///     );
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// If any of the expected fields are not present on the recorded
    /// event, the test will fail:
    ///
    /// ```should_panic
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event().with_fields(
    ///     expect::field("field")
    ///         .with_value(&"value")
    ///         .and(expect::field("another_field").with_value(&42))
    ///         .and(expect::field("a_third_field").with_value(&true)),
    /// );
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// tracing::subscriber::with_default(subscriber, || {
    ///     tracing::info!(
    ///         field = "value",
    ///         a_third_field = true,
    ///     );
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// [`ExpectedField::and`]: fn@crate::field::ExpectedField::and
    pub fn and(mut self, field: ExpectedField) -> Self {
        self.fields.insert(field.name, field.value);
        self
    }

    /// Indicates that no fields other than those specified should be
    /// expected.
    ///
    /// This method performs the same function as
    /// [`ExpectedField::only`], but applies in the case where there are
    /// multiple fields expected.
    ///
    /// # Examples
    ///
    /// The following test will pass, even though additional fields are
    /// recorded on the event.
    ///
    /// ```
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event().with_fields(
    ///     expect::field("field")
    ///         .with_value(&"value")
    ///         .and(expect::field("another_field").with_value(&42)),
    /// );
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// tracing::subscriber::with_default(subscriber, || {
    ///     tracing::info!(
    ///         field = "value",
    ///         another_field = 42,
    ///         a_third_field = true,
    ///     );
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    ///
    /// If we include `only` on the `ExpectedFields` then the test
    /// will fail:
    ///
    /// ```should_panic
    /// use better_tracing_mock::{expect, subscriber};
    ///
    /// let event = expect::event().with_fields(
    ///     expect::field("field")
    ///         .with_value(&"value")
    ///         .and(expect::field("another_field").with_value(&42))
    ///         .only(),
    /// );
    ///
    /// let (subscriber, handle) = subscriber::mock()
    ///     .event(event)
    ///     .run_with_handle();
    ///
    /// tracing::subscriber::with_default(subscriber, || {
    ///     tracing::info!(
    ///         field = "value",
    ///         another_field = 42,
    ///         a_third_field = true,
    ///     );
    /// });
    ///
    /// handle.assert_finished();
    /// ```
    pub fn only(self) -> Self {
        Self { only: true, ..self }
    }

    fn compare_or_panic(
        &mut self,
        name: &str,
        value: &dyn Value,
        ctx: &str,
        subscriber_name: &str,
    ) {
        let value = value.into();
        match self.fields.remove(name) {
            Some(ExpectedValue::Any) => {}
            Some(expected) => assert!(
                expected == value,
                "\n[{}] expected `{}` to contain:\n\t`{}{}`\nbut got:\n\t`{}{}`",
                subscriber_name,
                ctx,
                name,
                expected,
                name,
                value
            ),
            None if self.only => panic!(
                "[{}]expected `{}` to contain only:\n\t`{}`\nbut got:\n\t`{}{}`",
                subscriber_name, ctx, self, name, value
            ),
            _ => {}
        }
    }

    pub(crate) fn checker<'a>(
        &'a mut self,
        ctx: &'a str,
        subscriber_name: &'a str,
    ) -> CheckVisitor<'a> {
        CheckVisitor {
            expect: self,
            ctx,
            subscriber_name,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl fmt::Display for ExpectedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpectedValue::F64(v) => write!(f, "f64 = {:?}", v),
            ExpectedValue::I64(v) => write!(f, "i64 = {:?}", v),
            ExpectedValue::U64(v) => write!(f, "u64 = {:?}", v),
            ExpectedValue::Bool(v) => write!(f, "bool = {:?}", v),
            ExpectedValue::Str(v) => write!(f, "&str = {:?}", v),
            ExpectedValue::Debug(v) => write!(f, "&fmt::Debug = {:?}", v),
            ExpectedValue::Any => write!(f, "_ = _"),
        }
    }
}

pub(crate) struct CheckVisitor<'a> {
    expect: &'a mut ExpectedFields,
    ctx: &'a str,
    subscriber_name: &'a str,
}

impl Visit for CheckVisitor<'_> {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.expect
            .compare_or_panic(field.name(), &value, self.ctx, self.subscriber_name)
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.expect
            .compare_or_panic(field.name(), &value, self.ctx, self.subscriber_name)
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.expect
            .compare_or_panic(field.name(), &value, self.ctx, self.subscriber_name)
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.expect
            .compare_or_panic(field.name(), &value, self.ctx, self.subscriber_name)
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.expect
            .compare_or_panic(field.name(), &value, self.ctx, self.subscriber_name)
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.expect.compare_or_panic(
            field.name(),
            &field::debug(value),
            self.ctx,
            self.subscriber_name,
        )
    }
}

impl CheckVisitor<'_> {
    pub(crate) fn finish(self) {
        assert!(
            self.expect.fields.is_empty(),
            "[{}] {}missing {}",
            self.subscriber_name,
            self.expect,
            self.ctx
        );
    }
}

impl<'a> From<&'a dyn Value> for ExpectedValue {
    fn from(value: &'a dyn Value) -> Self {
        struct MockValueBuilder {
            value: Option<ExpectedValue>,
        }

        impl Visit for MockValueBuilder {
            fn record_f64(&mut self, _: &Field, value: f64) {
                self.value = Some(ExpectedValue::F64(value));
            }

            fn record_i64(&mut self, _: &Field, value: i64) {
                self.value = Some(ExpectedValue::I64(value));
            }

            fn record_u64(&mut self, _: &Field, value: u64) {
                self.value = Some(ExpectedValue::U64(value));
            }

            fn record_bool(&mut self, _: &Field, value: bool) {
                self.value = Some(ExpectedValue::Bool(value));
            }

            fn record_str(&mut self, _: &Field, value: &str) {
                self.value = Some(ExpectedValue::Str(value.to_owned()));
            }

            fn record_debug(&mut self, _: &Field, value: &dyn fmt::Debug) {
                self.value = Some(ExpectedValue::Debug(format!("{:?}", value)));
            }
        }

        let fake_field = callsite!(name: "fake", kind: Kind::EVENT, fields: fake_field)
            .metadata()
            .fields()
            .field("fake_field")
            .unwrap();
        let mut builder = MockValueBuilder { value: None };
        value.record(&fake_field, &mut builder);
        builder
            .value
            .expect("finish called before a value was recorded")
    }
}

impl fmt::Display for ExpectedFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fields ")?;
        let entries = self
            .fields
            .iter()
            .map(|(k, v)| (field::display(k), field::display(v)));
        f.debug_map().entries(entries).finish()
    }
}
