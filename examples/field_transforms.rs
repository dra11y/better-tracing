//! Field transformation example.
//!
//! This example demonstrates the field transformation layer that allows you to:
//! - Rename fields from third-party crates
//! - Hide sensitive or noisy fields
//! - Truncate long values
//! - Add prefixes for visual identification
//! - Apply custom transformations
//!
//! Run with:
//!   cargo run --example field_transforms

use tracing::{info, span, Level};
use tracing_subscriber::{
    layer::{transform::FieldTransformLayer, SubscriberExt},
    registry::Registry,
    util::SubscriberInitExt,
};

/// Simulate a third-party crate like kube-rs
mod kube_client {
    use tracing::{info, instrument};

    #[instrument]
    pub fn create_pod(
        resource_name: &str,
        namespace: &str,
        uid: &str,
        internal_token: &str,
        status: &str,
    ) {
        info!(
            resource_name,
            namespace, uid, internal_token, status, "Creating Kubernetes pod"
        );
    }

    #[instrument]
    pub fn watch_deployment(deployment_name: &str, resource_version: &str, phase: &str) {
        info!(
            deployment_name,
            resource_version, phase, "Watching deployment for changes"
        );
    }
}

/// Simulate HTTP client logging
mod http_client {
    use tracing::{info, instrument};

    #[instrument]
    pub fn make_request(method: &str, url: &str, status: u16, duration_ms: f64) {
        info!(method, url, status, duration_ms, "HTTP request completed");
    }
}

fn main() {
    // Set up field transformations for different third-party crates
    let transform_layer = FieldTransformLayer::new()
        // Transform Kubernetes-related logs
        .with_target_transform("kube_client", |builder| {
            builder
                .rename_field("resource_name", "k8s_resource") // More readable field name
                .rename_field("namespace", "ns") // Shorter field name
                .hide_field("internal_token") // Hide sensitive data
                .hide_field("resource_version") // Hide noisy internal field
                .truncate_field("uid", 8) // Truncate long UIDs
                .prefix_field("status", "🎯") // Add visual indicator
                .transform_field("phase", |phase| {
                    // Custom status transformation
                    match phase.trim_matches('"') {
                        "Running" => "✅ Running".to_string(),
                        "Pending" => "🟡 Pending".to_string(),
                        "Failed" => "❌ Failed".to_string(),
                        other => other.to_string(),
                    }
                })
        })
        // Transform HTTP client logs
        .with_target_transform("http_client", |builder| {
            builder
                .rename_field("method", "http_method") // Clearer field name
                .truncate_field("url", 60) // Limit URL length
                .transform_field("status", |status| {
                    // Color-code HTTP status
                    match status.parse::<u16>().unwrap_or(0) {
                        200..=299 => format!("✅ {}", status),
                        400..=499 => format!("⚠️ {}", status),
                        500..=599 => format!("🔥 {}", status),
                        _ => status.to_string(),
                    }
                })
                .transform_field("duration_ms", |duration| {
                    // Performance indicators
                    match duration.parse::<f64>().unwrap_or(0.0) {
                        d if d < 100.0 => format!("⚡ {:.1}ms", d),
                        d if d < 1000.0 => format!("🟡 {:.1}ms", d),
                        d => format!("🔴 {:.1}ms", d),
                    }
                })
        });

    // Set up the subscriber with field transformations
    Registry::default()
        .with(transform_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_ansi(true),
        )
        .init();

    println!("=== Field Transformation Example ===\n");

    info!("Application starting with field transformations enabled");

    // Example 1: Kubernetes operations (fields will be transformed)
    let span = span!(Level::INFO, "k8s_operations");
    let _guard = span.enter();

    kube_client::create_pod(
        "my-application-pod-12345",
        "production",
        "550e8400-e29b-41d4-a716-446655440000", // Long UUID that will be truncated
        "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...", // Token that will be hidden
        "Running",
    );

    kube_client::watch_deployment(
        "my-app-deployment",
        "123456789", // Will be hidden
        "Pending",   // Will be transformed with emoji
    );

    drop(_guard);

    // Example 2: HTTP requests (different transformations)
    let span = span!(Level::INFO, "http_requests");
    let _guard = span.enter();

    http_client::make_request(
        "GET",
        "https://api.example.com/users/12345/profile/settings/preferences/notifications", // Long URL
        200,
        45.2, // Fast response
    );

    http_client::make_request(
        "POST",
        "https://api.example.com/auth/login",
        401,
        120.5, // Slow response
    );

    http_client::make_request(
        "PUT",
        "https://api.example.com/data/upload",
        500,
        2340.8, // Very slow response
    );

    drop(_guard);

    // Example 3: Regular application logs (no transformations)
    let span = span!(Level::INFO, "app_logic");
    let _guard = span.enter();

    info!(
        user_id = 12345,
        operation = "user_update",
        success = true,
        "User operation completed"
    );

    info!("Application shutting down");
}
