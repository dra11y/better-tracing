//! Third-party crate field transformation example.
//!
//! This example shows how to handle real-world scenarios with popular crates
//! that generate noisy or poorly formatted logs.
//!
//! Run with:
//!   cargo run --example third_party_crates

use tracing::{info, span, Level};
use tracing_subscriber::{
    layer::{transform::FieldTransformLayer, SubscriberExt},
    registry::Registry,
    util::SubscriberInitExt,
};

/// Simulate reqwest HTTP client logs
mod reqwest_simulation {
    use tracing::{debug, info};

    pub fn get(url: &str) {
        debug!(target: "reqwest::client", method = "GET", uri = url, version = "HTTP/1.1", "sending request");
        info!(target: "reqwest::client", method = "GET", uri = url, status = 200, "request completed");
    }

    pub fn post_json(url: &str, body_size: usize) {
        debug!(target: "reqwest::client", method = "POST", uri = url, content_type = "application/json", body_length = body_size, "sending request");
        info!(target: "reqwest::client", method = "POST", uri = url, status = 201, "request completed");
    }
}

/// Simulate sqlx database logs
mod sqlx_simulation {
    use tracing::{debug, info};

    pub fn execute_query(sql: &str, rows_affected: u64, duration_ms: f64) {
        debug!(target: "sqlx::query", query = sql, "executing query");
        info!(
            target: "sqlx::query",
            query = sql,
            rows_affected,
            duration_ms,
            pool_id = "pool_12345",
            connection_id = "conn_67890",
            "query executed"
        );
    }
}

/// Simulate tokio runtime logs
mod tokio_simulation {
    use tracing::{debug, info};

    pub fn spawn_task() {
        debug!(target: "tokio::runtime", task_id = 42, worker_id = 1, "spawning task");
        info!(target: "tokio::runtime", task_id = 42, task_name = "my_async_task", "task completed");
    }
}

/// Simulate serde_json logs
mod serde_simulation {
    use tracing::{debug, warn};

    pub fn parse_json(json_str: &str) {
        if json_str.contains("invalid") {
            warn!(
                target: "serde_json",
                input = json_str,
                error = "missing field `required_field`",
                line = 1,
                column = 45,
                "JSON parsing failed"
            );
        } else {
            debug!(target: "serde_json", input = json_str, size = json_str.len(), "parsing JSON");
        }
    }
}

fn main() {
    // Configure transformations for popular third-party crates
    let transform_layer = FieldTransformLayer::new()
        // reqwest HTTP client - clean up verbose logs
        .with_target_transform("reqwest", |builder| {
            builder
                .rename_field("uri", "url")
                .rename_field("method", "http_method")
                .hide_field("version") // HTTP/1.1 is noise
                .hide_field("content_type") // Usually obvious from context
                .truncate_field("url", 80) // Long URLs are noisy
                .transform_field("status", |status| {
                    match status.parse::<u16>().unwrap_or(0) {
                        200..=299 => format!("✅ {}", status),
                        400..=499 => format!("⚠️ {}", status),
                        500..=599 => format!("🚜 {}", status),
                        _ => status.to_string(),
                    }
                })
        })
        // sqlx database - focus on performance and results
        .with_target_transform("sqlx", |builder| {
            builder
                .rename_field("query", "sql")
                .hide_field("pool_id") // Internal detail
                .hide_field("connection_id") // Internal detail
                .truncate_field("sql", 120) // Long queries are noisy
                .prefix_field("rows_affected", "📊") // Visual indicator
                .transform_field("duration_ms", |duration| {
                    match duration.parse::<f64>().unwrap_or(0.0) {
                        d if d < 10.0 => format!("⚡ {:.1}ms", d),
                        d if d < 100.0 => format!("🟡 {:.1}ms", d),
                        d => format!("🔴 {:.1}ms", d),
                    }
                })
        })
        // tokio runtime - simplify task management logs
        .with_target_transform(
            "tokio",
            |builder| {
                builder
                    .rename_field("task_id", "task")
                    .rename_field("task_name", "name")
                    .rename_field("worker_id", "worker")
                    .prefix_field("name", "🚀")
            }, // Visual indicator for tasks
        )
        // serde_json - focus on errors, hide verbose success logs
        .with_target_transform(
            "serde_json",
            |builder| {
                builder
                    .hide_field("size") // Not usually important
                    .hide_field("line") // Error details are usually enough
                    .hide_field("column") // Error details are usually enough
                    .truncate_field("input", 100) // Truncate large JSON
                    .prefix_field("error", "❌")
            }, // Highlight errors
        );

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

    println!("=== Third-Party Crate Transformation Example ===\n");

    info!("Application starting with third-party crate transformations");

    // Example 1: HTTP requests with reqwest
    let span = span!(Level::INFO, "http_operations");
    let _guard = span.enter();

    reqwest_simulation::get(
        "https://api.github.com/users/octocat/repos?type=owner&sort=updated&per_page=50",
    );
    reqwest_simulation::post_json("https://api.example.com/webhooks", 1024);

    drop(_guard);

    // Example 2: Database operations with sqlx
    let span = span!(Level::INFO, "database_operations");
    let _guard = span.enter();

    sqlx_simulation::execute_query(
        "SELECT users.id, users.name, users.email, profiles.bio, profiles.avatar_url FROM users JOIN profiles ON users.id = profiles.user_id WHERE users.active = true AND users.created_at > $1 ORDER BY users.last_login DESC LIMIT 50",
        23,
        8.5
    );

    sqlx_simulation::execute_query(
        "UPDATE user_preferences SET theme = $1, notifications = $2 WHERE user_id = $3",
        1,
        145.2, // Slow update
    );

    drop(_guard);

    // Example 3: Async task management with tokio
    let span = span!(Level::INFO, "async_operations");
    let _guard = span.enter();

    tokio_simulation::spawn_task();

    drop(_guard);

    // Example 4: JSON parsing with serde
    let span = span!(Level::INFO, "json_operations");
    let _guard = span.enter();

    serde_simulation::parse_json(r#"{"name": "John", "age": 30, "city": "New York"}"#);
    serde_simulation::parse_json(r#"{"invalid": "missing required field"}"#);

    drop(_guard);

    info!("Application completed - check the logs above to see the transformations!");

    println!("\n=== Transformation Summary ===");
    println!("✅ reqwest: Cleaned URLs, added status emojis, hid HTTP version");
    println!("📊 sqlx: Truncated long queries, added performance colors, hid connection details");
    println!("🚀 tokio: Simplified task IDs, added task emojis");
    println!("❌ serde_json: Highlighted errors, truncated large JSON inputs");
}
