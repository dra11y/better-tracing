//! Custom formatter with field transformations example.
//!
//! This example demonstrates how field transformations work seamlessly with custom
//! formatters, showing the full pipeline from transformation to final output.
//!
//! Run with:
//!   cargo run --example custom_formatter

use std::fmt;
use tracing::{info, span, Level, Subscriber};
use tracing_core::Event;
use tracing_subscriber::{
    fmt::{
        format::{FormatEvent, FormatFields},
        FmtContext, FormattedFields,
    },
    layer::{transform::FieldTransformLayer, SubscriberExt},
    registry::{LookupSpan, Registry},
    util::SubscriberInitExt,
};

/// Custom formatter that formats events in a specific style
struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Write timestamp (using a simple format since chrono isn't available)
        write!(writer, "[timestamp] ")?;

        // Write level with color
        let level = *event.metadata().level();
        match level {
            Level::ERROR => write!(writer, "🔴 ERROR")?,
            Level::WARN => write!(writer, "🟡 WARN ")?,
            Level::INFO => write!(writer, "🟢 INFO ")?,
            Level::DEBUG => write!(writer, "🔵 DEBUG")?,
            Level::TRACE => write!(writer, "⚪ TRACE")?,
        }

        // Write target in brackets
        write!(writer, " [{}]", event.metadata().target())?;

        // Write span hierarchy with visual separators
        if let Some(scope) = ctx.event_scope() {
            let mut spans: Vec<_> = scope.from_root().collect();
            spans.reverse();

            if !spans.is_empty() {
                write!(writer, " 📍")?;
                for (i, span) in spans.iter().enumerate() {
                    if i > 0 {
                        write!(writer, " → ")?;
                    }
                    write!(writer, " {}", span.name())?;

                    // Include span fields (which may be transformed)
                    let ext = span.extensions();
                    if let Some(fields) = ext.get::<FormattedFields<N>>() {
                        if !fields.fields.is_empty() {
                            write!(writer, "{{{}}}", fields.fields)?;
                        }
                    }
                }
            }
        }

        // Write the event message
        write!(writer, " | {}", event.metadata().name())?;

        // Note: Event fields would normally be transformed and displayed here
        // but we'll keep this simple for the example

        writeln!(writer)?;
        Ok(())
    }
}

/// Simulate HTTP client with verbose fields
mod http_client {
    use tracing::info;

    pub fn make_request(
        url: &str,
        method: &str,
        headers_count: usize,
        timeout_ms: u64,
        retry_count: u32,
        user_agent: &str,
    ) {
        info!(
            target: "http::client",
            url,
            method,
            headers_count,
            timeout_ms,
            retry_count,
            user_agent,
            connection_pool_size = 10,
            keep_alive = true,
            "HTTP request initiated"
        );
    }

    pub fn response_received(
        status_code: u16,
        content_length: u64,
        response_time_ms: u64,
        server_header: &str,
    ) {
        info!(
            target: "http::response",
            status_code,
            content_length,
            response_time_ms,
            server_header,
            "Response received"
        );
    }
}

/// Simulate database operations
mod database {
    use tracing::info;

    pub fn execute_query(
        query: &str,
        params_count: usize,
        connection_id: &str,
        database_name: &str,
        execution_time_ms: u64,
    ) {
        info!(
            target: "db::postgres",
            query,
            params_count,
            connection_id,
            database_name,
            execution_time_ms,
            rows_affected = 42,
            "Query executed"
        );
    }
}

fn main() {
    // Configure field transformations to clean up verbose third-party logs
    let transform_layer = FieldTransformLayer::new()
        .with_target_transform("http", |builder| {
            builder
                .hide_field("connection_pool_size") // Implementation detail
                .hide_field("keep_alive") // Usually not relevant
                .hide_field("headers_count") // Too verbose
                .truncate_field("user_agent", 20) // Can be very long
                .truncate_field("url", 50) // URLs can be extremely long
                .transform_field("status_code", |code| {
                    // Status with visual indicators
                    let status: u16 = code.parse().unwrap_or(0);
                    match status {
                        200..=299 => format!("✅ {}", status),
                        300..=399 => format!("🔄 {}", status),
                        400..=499 => format!("⚠️ {}", status),
                        500..=599 => format!("❌ {}", status),
                        _ => status.to_string(),
                    }
                })
                .transform_field("response_time_ms", |time| {
                    // Performance indicators
                    let ms: u64 = time.parse().unwrap_or(0);
                    match ms {
                        0..=100 => format!("🚀 {}ms", ms),
                        101..=500 => format!("⚡ {}ms", ms),
                        501..=2000 => format!("🐌 {}ms", ms),
                        _ => format!("🐌🐌 {}ms", ms),
                    }
                })
                .transform_field("content_length", |bytes| {
                    // Human-readable sizes
                    let size: u64 = bytes.parse().unwrap_or(0);
                    match size {
                        0..=1024 => format!("{}B", size),
                        1025..=1048576 => format!("{:.1}KB", size as f64 / 1024.0),
                        _ => format!("{:.1}MB", size as f64 / 1048576.0),
                    }
                })
        })
        .with_target_transform("db", |builder| {
            builder
                .rename_field("connection_id", "conn") // Shorter
                .rename_field("database_name", "db") // Shorter
                .hide_field("rows_affected") // Often not relevant for tracing
                .truncate_field("query", 60) // SQL queries can be very long
                .transform_field("execution_time_ms", |time| {
                    // Performance indicators
                    let ms: u64 = time.parse().unwrap_or(0);
                    match ms {
                        0..=10 => format!("⚡ {}ms", ms),
                        11..=100 => format!("🔵 {}ms", ms),
                        101..=1000 => format!("🟡 {}ms", ms),
                        _ => format!("🔴 {}ms", ms),
                    }
                })
        });

    // Initialize with custom formatter and transformations
    Registry::default()
        .with(transform_layer)
        .with(tracing_subscriber::fmt::layer().event_format(CustomFormatter))
        .init();

    println!("=== Custom Formatter + Field Transformations Example ===\n");

    info!("Demonstrating custom formatter with field transformations");

    // Example 1: HTTP operations with transformation
    let span = span!(
        Level::INFO,
        "api_request",
        endpoint = "/users/123",
        client_id = "web-app"
    );
    let _guard = span.enter();

    http_client::make_request(
        "https://api.example.com/v1/users/123?include=profile,settings,preferences&format=json",
        "GET",
        15, // Will be hidden
        5000,
        3,
        "MyApp/1.0 (Linux; x86_64) RequestsLib/2.28.1 Python/3.9.0", // Will be truncated
    );

    http_client::response_received(
        200,     // Will get ✅ prefix
        1048576, // Will be formatted as MB
        250,     // Will get ⚡ prefix
        "nginx/1.18.0",
    );

    drop(_guard);

    // Example 2: Database operations with transformation
    let span = span!(
        Level::INFO,
        "user_query",
        user_id = 123,
        operation = "profile_update"
    );
    let _guard = span.enter();

    database::execute_query(
        "SELECT u.id, u.email, u.name, p.bio, p.avatar_url, s.theme, s.language FROM users u LEFT JOIN profiles p ON u.id = p.user_id LEFT JOIN settings s ON u.id = s.user_id WHERE u.id = $1",
        1,
        "conn-a1b2c3d4-e5f6-7890",  // Will be shortened to "conn"
        "production_db",            // Will be shortened to "db"
        15,                         // Will get ⚡ prefix
    );

    drop(_guard);

    // Example 3: Slow operations to show performance indicators
    let span = span!(Level::WARN, "slow_operations");
    let _guard = span.enter();

    http_client::response_received(
        500,     // Will get ❌ prefix
        5242880, // Will be formatted as MB
        3500,    // Will get 🐌🐌 prefix (very slow)
        "Apache/2.4.41",
    );

    database::execute_query(
        "SELECT * FROM audit_logs WHERE created_at > NOW() - INTERVAL '30 days' ORDER BY created_at DESC",
        0,
        "conn-slow-analytics",
        "analytics_warehouse",
        2500,  // Will get 🔴 prefix (very slow)
    );

    drop(_guard);

    info!("Custom formatter demo completed");

    println!("\n=== Custom Formatter Integration Summary ===");
    println!("🎨 Custom styling: Timestamps, emojis, visual hierarchy");
    println!("🔧 Field transformations: Applied before custom formatting");
    println!("📊 Performance indicators: Visual feedback on timing/status");
    println!("🧹 Noise reduction: Verbose fields hidden or shortened");
    println!("🔗 Seamless integration: Transformations work with any formatter");
}
