//! Kubernetes field transformation example.
//!
//! This example demonstrates specific transformations for Kubernetes/container workloads,
//! showing how to make kube-rs and similar crate logs much more readable and actionable.
//!
//! Run with:
//!   cargo run --example kubernetes

use tracing::{info, span, Level};
use tracing_subscriber::{
    layer::{transform::FieldTransformLayer, SubscriberExt},
    registry::Registry,
    util::SubscriberInitExt,
};

/// Simulate kube-rs API calls
mod kube {
    use tracing::info;

    pub fn create_deployment(
        name: &str,
        namespace: &str,
        image: &str,
        replicas: u32,
        resource_version: &str,
        uid: &str,
    ) {
        info!(
            target: "kube::api",
            resource_name = name,
            namespace,
            image,
            replicas,
            resource_version,
            uid,
            api_version = "apps/v1",
            kind = "Deployment",
            "Creating deployment"
        );
    }

    pub fn watch_pods(namespace: &str, label_selector: &str) {
        info!(
            target: "kube::watch",
            namespace,
            label_selector,
            resource_version = "123456",
            timeout_seconds = 300,
            "Starting pod watch"
        );
    }

    pub fn pod_status_update(
        pod_name: &str,
        namespace: &str,
        phase: &str,
        container_statuses: &str,
        uid: &str,
    ) {
        info!(
            target: "kube::events",
            resource_name = pod_name,
            namespace,
            phase,
            container_statuses,
            uid,
            resource_version = "987654",
            "Pod status changed"
        );
    }
}

/// Simulate container runtime logs
mod containerd {
    use tracing::info;

    pub fn pull_image(image: &str, size_bytes: u64) {
        info!(
            target: "containerd::client",
            image,
            size_bytes,
            registry = "docker.io",
            "Image pulled successfully"
        );
    }

    pub fn start_container(container_id: &str, image: &str, command: &str, working_dir: &str) {
        info!(
            target: "containerd::runtime",
            container_id,
            image,
            command,
            working_dir,
            runtime = "runc",
            "Container started"
        );
    }
}

fn main() {
    // Configure transformations specifically for Kubernetes ecosystem
    let transform_layer = FieldTransformLayer::new()
        // kube-rs API operations
        .with_target_transform("kube", |builder| {
            builder
                .rename_field("resource_name", "name") // Shorter, clearer
                .rename_field("namespace", "ns") // Kubernetes shorthand
                .rename_field("api_version", "api") // Shorter
                .hide_field("resource_version") // Usually not relevant for humans
                .hide_field("timeout_seconds") // Implementation detail
                .truncate_field("uid", 8) // UUIDs are long and noisy
                .truncate_field("label_selector", 40) // Can be very long
                .prefix_field("kind", "📦") // Visual indicator for resource type
                .transform_field("phase", |phase| {
                    // Pod phases with visual status
                    match phase.trim_matches('"') {
                        "Pending" => "🟡 Pending".to_string(),
                        "Running" => "🟢 Running".to_string(),
                        "Succeeded" => "✅ Succeeded".to_string(),
                        "Failed" => "❌ Failed".to_string(),
                        "Unknown" => "❓ Unknown".to_string(),
                        other => other.to_string(),
                    }
                })
                .transform_field("replicas", |replicas| {
                    // Add visual indicator for scale
                    let count: u32 = replicas.parse().unwrap_or(0);
                    match count {
                        0 => "⭕ 0".to_string(),
                        1 => "1️⃣ 1".to_string(),
                        2..=5 => format!("🔢 {}", count),
                        _ => format!("🚀 {}", count),
                    }
                })
        })
        // Container runtime operations
        .with_target_transform("containerd", |builder| {
            builder
                .rename_field("container_id", "ctr_id") // Shorter
                .truncate_field("ctr_id", 12) // Docker-style short IDs
                .hide_field("runtime") // Usually runc, not interesting
                .hide_field("registry") // Usually obvious from image name
                .prefix_field("image", "🐳") // Docker/container indicator
                .transform_field("size_bytes", |bytes| {
                    // Human-readable sizes
                    let size: u64 = bytes.parse().unwrap_or(0);
                    match size {
                        0..=1024 => format!("📄 {}B", size),
                        1025..=1048576 => format!("📄 {:.1}KB", size as f64 / 1024.0),
                        1048577..=1073741824 => format!("📦 {:.1}MB", size as f64 / 1048576.0),
                        _ => format!("📦 {:.1}GB", size as f64 / 1073741824.0),
                    }
                })
        });

    // Initialize the subscriber
    Registry::default()
        .with(transform_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_ansi(true),
        )
        .init();

    println!("=== Kubernetes Field Transformation Example ===\n");

    info!("Starting Kubernetes operations demo");

    // Example 1: Deployment lifecycle
    let span = span!(Level::INFO, "deployment_operations");
    let _guard = span.enter();

    kube::create_deployment(
        "web-app-frontend",
        "production",
        "nginx:1.21-alpine",
        3,
        "12345",                                // Will be hidden
        "550e8400-e29b-41d4-a716-446655440000", // Will be truncated
    );

    drop(_guard);

    // Example 2: Pod monitoring
    let span = span!(Level::INFO, "pod_monitoring");
    let _guard = span.enter();

    kube::watch_pods(
        "production",
        "app=web-frontend,version=v1.2.3,tier=frontend,environment=prod", // Long selector
    );

    kube::pod_status_update(
        "web-app-frontend-7d4b8c6f9-xyz12",
        "production",
        "Running",
        r#"[{"name":"nginx","ready":true,"state":{"running":{"startedAt":"2024-08-02T10:00:00Z"}}}]"#,
        "a1b2c3d4-e5f6-7890-1234-567890abcdef",
    );

    kube::pod_status_update(
        "web-app-frontend-7d4b8c6f9-abc34",
        "production",
        "Failed",
        r#"[{"name":"nginx","ready":false,"state":{"terminated":{"reason":"Error","exitCode":1}}}]"#,
        "f1e2d3c4-b5a6-9087-6543-210987654321",
    );

    drop(_guard);

    // Example 3: Container operations
    let span = span!(Level::INFO, "container_operations");
    let _guard = span.enter();

    containerd::pull_image("nginx:1.21-alpine", 5242880); // 5MB
    containerd::pull_image("postgres:13", 314572800); // 300MB

    containerd::start_container(
        "a1b2c3d4e5f67890123456789abcdef0", // Long container ID
        "nginx:1.21-alpine",
        "/docker-entrypoint.sh nginx -g daemon off;",
        "/usr/share/nginx/html",
    );

    drop(_guard);

    info!("Kubernetes operations demo completed");

    println!("\n=== K8s Transformation Summary ===");
    println!("🎯 Kubernetes API: Shortened field names, hid internal IDs, added status emojis");
    println!("🐳 Container runtime: Truncated IDs, human-readable sizes, focused on essentials");
    println!("📦 Resource types: Visual indicators for different Kubernetes resources");
    println!("🟢 Pod phases: Color-coded status with emojis for quick visual scanning");
}
