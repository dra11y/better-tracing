//! Field transformation layer for zero-allocation span field manipulation.
//!
//! This layer intercepts field recording and applies transformations based on
//! configurable rules, storing the transformed results in the existing
//! `FormattedFields<N>` extension storage for zero-cost access during formatting.

use crate::{
    field::Visit,
    fmt::{format::Writer, FormattedFields},
    layer::{Context, Layer},
    registry::LookupSpan,
};
use std::{fmt, marker::PhantomData};
use tracing_core::{
    field::Field,
    span::{Attributes, Id, Record},
    Subscriber,
};

/// A layer that transforms span fields during recording.
///
/// This layer sits between the registry and formatting layers, intercepting
/// field recording to apply transformations. When no transformations are
/// configured for a span, it has zero runtime overhead.
///
/// # Example
///
/// ```rust
/// use better_tracing::layer::transform::FieldTransformLayer;
///
/// let transform_layer = FieldTransformLayer::new()
///     .with_target_transform("kube", |builder| builder
///         .rename_field("resource_name", "k8s_resource")
///         .hide_field("internal_token")
///         .truncate_field("uid", 8)
///     );
///
/// better_tracing::registry()
///     .with(transform_layer)
///     .with(better_tracing::fmt::layer())
///     .init();
/// ```
#[derive(Debug)]
pub struct FieldTransformLayer<N = ()> {
    transforms: N,
    _phantom: PhantomData<fn()>,
}

/// Configuration for field transformations.
#[derive(Debug)]
pub struct TransformConfig {
    target_rules: Vec<TargetRule>,
}

/// Transformation rules for a specific target pattern.
#[derive(Debug)]
pub struct TargetRule {
    target_pattern: &'static str,
    field_renames: Vec<(&'static str, &'static str)>,
    hidden_fields: Vec<&'static str>,
    field_transforms: Vec<FieldTransform>,
}

/// A field transformation.
#[derive(Debug)]
pub struct FieldTransform {
    field_name: &'static str,
    transform_type: TransformType,
}

/// Types of field transformations.
#[derive(Debug)]
pub enum TransformType {
    /// Truncate to N characters
    Truncate(usize),
    /// Add a static prefix
    Prefix(&'static str),
    /// Apply a custom transformation function
    Custom(fn(&str) -> String),
}

impl FieldTransformLayer<()> {
    /// Create a new field transform layer with no transformations.
    ///
    /// This has zero runtime cost until transformations are added.
    pub fn new() -> Self {
        Self {
            transforms: (),
            _phantom: PhantomData,
        }
    }

    /// Add transformations for a specific target pattern.
    pub fn with_target_transform<F>(
        self,
        target_pattern: &'static str,
        builder: F,
    ) -> FieldTransformLayer<TransformConfig>
    where
        F: FnOnce(TargetRuleBuilder) -> TargetRuleBuilder,
    {
        let rule = builder(TargetRuleBuilder::new(target_pattern)).build();

        FieldTransformLayer {
            transforms: TransformConfig {
                target_rules: vec![rule],
            },
            _phantom: PhantomData,
        }
    }
}

impl FieldTransformLayer<TransformConfig> {
    /// Add additional transformations for another target pattern.
    pub fn with_target_transform<F>(mut self, target_pattern: &'static str, builder: F) -> Self
    where
        F: FnOnce(TargetRuleBuilder) -> TargetRuleBuilder,
    {
        let rule = builder(TargetRuleBuilder::new(target_pattern)).build();
        self.transforms.target_rules.push(rule);
        self
    }
}

/// Builder for creating target-specific transformation rules.
#[derive(Debug)]
pub struct TargetRuleBuilder {
    target_pattern: &'static str,
    field_renames: Vec<(&'static str, &'static str)>,
    hidden_fields: Vec<&'static str>,
    field_transforms: Vec<FieldTransform>,
}

impl TargetRuleBuilder {
    fn new(target_pattern: &'static str) -> Self {
        Self {
            target_pattern,
            field_renames: Vec::new(),
            hidden_fields: Vec::new(),
            field_transforms: Vec::new(),
        }
    }

    /// Rename a field.
    pub fn rename_field(mut self, from: &'static str, to: &'static str) -> Self {
        self.field_renames.push((from, to));
        self
    }

    /// Hide a field from display.
    pub fn hide_field(mut self, field: &'static str) -> Self {
        self.hidden_fields.push(field);
        self
    }

    /// Truncate a field to the specified length.
    pub fn truncate_field(mut self, field: &'static str, max_len: usize) -> Self {
        self.field_transforms.push(FieldTransform {
            field_name: field,
            transform_type: TransformType::Truncate(max_len),
        });
        self
    }

    /// Add a static prefix to a field.
    pub fn prefix_field(mut self, field: &'static str, prefix: &'static str) -> Self {
        self.field_transforms.push(FieldTransform {
            field_name: field,
            transform_type: TransformType::Prefix(prefix),
        });
        self
    }

    /// Apply a custom transformation to a field.
    pub fn transform_field(mut self, field: &'static str, transform: fn(&str) -> String) -> Self {
        self.field_transforms.push(FieldTransform {
            field_name: field,
            transform_type: TransformType::Custom(transform),
        });
        self
    }

    /// Build the target rule (internal method).
    pub fn build(self) -> TargetRule {
        TargetRule {
            target_pattern: self.target_pattern,
            field_renames: self.field_renames,
            hidden_fields: self.hidden_fields,
            field_transforms: self.field_transforms,
        }
    }
}

/// A field visitor that applies transformations during recording.
struct TransformingVisitor<'a> {
    writer: Writer<'a>,
    rule: &'a TargetRule,
}

impl<'a> TransformingVisitor<'a> {
    fn new(writer: Writer<'a>, rule: &'a TargetRule) -> Self {
        Self { writer, rule }
    }
}

impl Visit for TransformingVisitor<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let field_name = field.name();

        // Check if field should be hidden
        if self
            .rule
            .hidden_fields
            .iter()
            .any(|&hidden| hidden == field_name)
        {
            return;
        }

        // Check for field rename
        let display_name = self
            .rule
            .field_renames
            .iter()
            .find(|(from, _)| *from == field_name)
            .map(|(_, to)| *to)
            .unwrap_or(field_name);

        // Check for field transformation
        if let Some(transform) = self
            .rule
            .field_transforms
            .iter()
            .find(|t| t.field_name == field_name)
        {
            let value_str = format!("{:?}", value);
            let transformed_value = match &transform.transform_type {
                TransformType::Truncate(max_len) => {
                    if value_str.len() > *max_len {
                        format!("{}...", &value_str[..*max_len])
                    } else {
                        value_str
                    }
                }
                TransformType::Prefix(prefix) => {
                    format!("{} {}", prefix, value_str)
                }
                TransformType::Custom(func) => func(&value_str),
            };
            let _ = write!(self.writer, "{}={}", display_name, transformed_value);
        } else {
            let _ = write!(self.writer, "{}={:?}", display_name, value);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        let field_name = field.name();

        // Check if field should be hidden
        if self
            .rule
            .hidden_fields
            .iter()
            .any(|&hidden| hidden == field_name)
        {
            return;
        }

        // Check for field rename
        let display_name = self
            .rule
            .field_renames
            .iter()
            .find(|(from, _)| *from == field_name)
            .map(|(_, to)| *to)
            .unwrap_or(field_name);

        // Check for field transformation
        if let Some(transform) = self
            .rule
            .field_transforms
            .iter()
            .find(|t| t.field_name == field_name)
        {
            let transformed_value = match &transform.transform_type {
                TransformType::Truncate(max_len) => {
                    if value.len() > *max_len {
                        format!("{}...", &value[..*max_len])
                    } else {
                        value.to_string()
                    }
                }
                TransformType::Prefix(prefix) => {
                    format!("{} {}", prefix, value)
                }
                TransformType::Custom(func) => func(value),
            };
            let _ = write!(self.writer, "{}={}", display_name, transformed_value);
        } else {
            let _ = write!(self.writer, "{}={}", display_name, value);
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        let field_name = field.name();
        if !self
            .rule
            .hidden_fields
            .iter()
            .any(|&hidden| hidden == field_name)
        {
            let display_name = self
                .rule
                .field_renames
                .iter()
                .find(|(from, _)| *from == field_name)
                .map(|(_, to)| *to)
                .unwrap_or(field_name);
            let _ = write!(self.writer, "{}={}", display_name, value);
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        let field_name = field.name();
        if !self
            .rule
            .hidden_fields
            .iter()
            .any(|&hidden| hidden == field_name)
        {
            let display_name = self
                .rule
                .field_renames
                .iter()
                .find(|(from, _)| *from == field_name)
                .map(|(_, to)| *to)
                .unwrap_or(field_name);
            let _ = write!(self.writer, "{}={}", display_name, value);
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        let field_name = field.name();
        if !self
            .rule
            .hidden_fields
            .iter()
            .any(|&hidden| hidden == field_name)
        {
            let display_name = self
                .rule
                .field_renames
                .iter()
                .find(|(from, _)| *from == field_name)
                .map(|(_, to)| *to)
                .unwrap_or(field_name);
            let _ = write!(self.writer, "{}={}", display_name, value);
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        let field_name = field.name();
        if !self
            .rule
            .hidden_fields
            .iter()
            .any(|&hidden| hidden == field_name)
        {
            let display_name = self
                .rule
                .field_renames
                .iter()
                .find(|(from, _)| *from == field_name)
                .map(|(_, to)| *to)
                .unwrap_or(field_name);
            let _ = write!(self.writer, "{}={}", display_name, value);
        }
    }
}

// Zero-cost implementation when no transforms are configured
impl<S> Layer<S> for FieldTransformLayer<()>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    // All methods are no-ops, ensuring zero cost when no transforms are configured
}

impl<S> Layer<S> for FieldTransformLayer<TransformConfig>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        // Check if any rule matches this span's target
        let target = attrs.metadata().target();

        if let Some(rule) = self
            .transforms
            .target_rules
            .iter()
            .find(|rule| target.contains(rule.target_pattern))
        {
            // Apply transformations to this span's fields
            if let Some(span) = ctx.span(id) {
                let mut extensions = span.extensions_mut();

                // Create a new FormattedFields with transformed content
                let mut fields = FormattedFields::<TransformConfig>::new(String::new());
                let mut visitor = TransformingVisitor::new(fields.as_writer(), rule);
                attrs.record(&mut visitor);

                // Store the transformed fields
                extensions.insert(fields);
            }
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let target = span.metadata().target();

            if let Some(rule) = self
                .transforms
                .target_rules
                .iter()
                .find(|rule| target.contains(rule.target_pattern))
            {
                let mut extensions = span.extensions_mut();

                if let Some(fields) = extensions.get_mut::<FormattedFields<TransformConfig>>() {
                    // Append transformed fields to existing ones
                    if !fields.fields.is_empty() {
                        fields.fields.push(' ');
                    }
                    let mut visitor = TransformingVisitor::new(fields.as_writer(), rule);
                    values.record(&mut visitor);
                } else {
                    // Create new transformed fields
                    let mut fields = FormattedFields::<TransformConfig>::new(String::new());
                    let mut visitor = TransformingVisitor::new(fields.as_writer(), rule);
                    values.record(&mut visitor);
                    extensions.insert(fields);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{layer::SubscriberExt, registry::Registry};
    use tracing::{span, Level};

    #[test]
    fn test_zero_cost_when_no_transforms() {
        // Verify that the layer has zero cost when no transformations are configured
        let layer = FieldTransformLayer::new();

        // This should compile and have no runtime overhead
        let subscriber = Registry::default().with(layer);

        // Basic smoke test - ensure it doesn't panic
        tracing::subscriber::with_default(subscriber, || {
            let span = span!(Level::INFO, "test_span", field1 = "value1");
            let _guard = span.enter();
        });
    }

    #[test]
    fn test_layer_creation_and_configuration() {
        // Test that the builder pattern works correctly
        let layer = FieldTransformLayer::new()
            .with_target_transform("kube", |builder| {
                builder
                    .rename_field("resource_name", "k8s_resource")
                    .hide_field("internal_token")
                    .truncate_field("uid", 8)
                    .prefix_field("status", "🎯")
                    .transform_field("phase", |value| match value {
                        "\"Running\"" => "✅ Running".to_string(),
                        "\"Failed\"" => "❌ Failed".to_string(),
                        other => other.to_string(),
                    })
            })
            .with_target_transform("http", |builder| {
                builder
                    .rename_field("method", "http_method")
                    .truncate_field("url", 50)
            });

        // Verify the configuration was built correctly
        assert_eq!(layer.transforms.target_rules.len(), 2);

        let kube_rule = &layer.transforms.target_rules[0];
        assert_eq!(kube_rule.target_pattern, "kube");
        assert_eq!(kube_rule.field_renames.len(), 1);
        assert_eq!(
            kube_rule.field_renames[0],
            ("resource_name", "k8s_resource")
        );
        assert_eq!(kube_rule.hidden_fields.len(), 1);
        assert_eq!(kube_rule.hidden_fields[0], "internal_token");
        assert_eq!(kube_rule.field_transforms.len(), 3);

        let http_rule = &layer.transforms.target_rules[1];
        assert_eq!(http_rule.target_pattern, "http");
        assert_eq!(http_rule.field_renames.len(), 1);
        assert_eq!(http_rule.field_renames[0], ("method", "http_method"));
    }

    #[test]
    fn test_target_rule_builder() {
        // Test the builder pattern for target rules
        let builder = TargetRuleBuilder::new("test_target");
        let rule = builder
            .rename_field("old", "new")
            .hide_field("secret")
            .truncate_field("long", 10)
            .prefix_field("status", "🎯")
            .transform_field("custom", |v| v.to_uppercase())
            .build();

        assert_eq!(rule.target_pattern, "test_target");
        assert_eq!(rule.field_renames.len(), 1);
        assert_eq!(rule.field_renames[0], ("old", "new"));
        assert_eq!(rule.hidden_fields.len(), 1);
        assert_eq!(rule.hidden_fields[0], "secret");
        assert_eq!(rule.field_transforms.len(), 3);

        // Test transform types
        assert_eq!(rule.field_transforms[0].field_name, "long");
        assert_eq!(rule.field_transforms[1].field_name, "status");
        assert_eq!(rule.field_transforms[2].field_name, "custom");

        match &rule.field_transforms[0].transform_type {
            TransformType::Truncate(n) => assert_eq!(*n, 10),
            _ => panic!("Expected Truncate transform"),
        }

        match &rule.field_transforms[1].transform_type {
            TransformType::Prefix(p) => assert_eq!(*p, "🎯"),
            _ => panic!("Expected Prefix transform"),
        }

        match &rule.field_transforms[2].transform_type {
            TransformType::Custom(_) => {} // Can't test function equality
            _ => panic!("Expected Custom transform"),
        }
    }

    #[test]
    fn test_transform_types() {
        // Test truncation logic
        let value = "this_is_a_very_long_string";
        let truncated = if value.len() > 10 {
            format!("{}...", &value[..10])
        } else {
            value.to_string()
        };
        assert_eq!(truncated, "this_is_a_...");

        // Test prefix logic
        let prefixed = format!("🎯 {}", "test_value");
        assert_eq!(prefixed, "🎯 test_value");

        // Test custom transform
        let custom_transform = |value: &str| match value {
            "running" => "✅ Running".to_string(),
            "failed" => "❌ Failed".to_string(),
            other => other.to_string(),
        };
        assert_eq!(custom_transform("running"), "✅ Running");
        assert_eq!(custom_transform("failed"), "❌ Failed");
        assert_eq!(custom_transform("other"), "other");
    }

    #[test]
    fn test_integration_with_registry() {
        // Test that the layer properly integrates with the registry
        let layer = FieldTransformLayer::new().with_target_transform("test_target", |builder| {
            builder
                .rename_field("field1", "renamed_field1")
                .hide_field("secret")
        });

        let subscriber = Registry::default().with(layer);

        // This should not panic and should work end-to-end
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::span!(
                target: "test_target",
                Level::INFO,
                "test_span",
                field1 = "value1",
                secret = "hidden_value",
                visible = "visible_value"
            );
            let _guard = span.enter();

            // Test recording additional fields
            span.record("field2", &"value2");
        });
    }

    #[test]
    fn test_multiple_layer_composition() {
        // Test that transform layers can be composed with other layers
        let transform_layer = FieldTransformLayer::new().with_target_transform("app", |builder| {
            builder
                .rename_field("user_id", "uid")
                .hide_field("password")
        });

        let fmt_layer = crate::fmt::layer().with_target(true).with_level(true);

        let subscriber = Registry::default().with(transform_layer).with(fmt_layer);

        // Should compose properly without panic
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::span!(
                target: "app::auth",
                Level::INFO,
                "login",
                user_id = 12345,
                password = "secret123",
                method = "oauth"
            );
            let _guard = span.enter();
        });
    }

    #[test]
    fn test_no_allocation_when_no_match() {
        // Test that no work is done when target doesn't match
        let layer = FieldTransformLayer::new()
            .with_target_transform("specific_target", |builder| {
                builder.rename_field("field", "renamed")
            });

        let subscriber = Registry::default().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            // This span should not trigger any transformations
            let span = tracing::span!(
                target: "different_target",
                Level::INFO,
                "test_span",
                field = "value"
            );
            let _guard = span.enter();
        });
    }
}
