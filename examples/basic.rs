//! Basic better-tracing usage example.
//!
//! This example demonstrates the fundamental setup and usage of better-tracing,
//! showing the "just works" experience compared to standard tracing-subscriber.
//!
//! Run with:
//!   cargo run --example basic

use tracing::{debug, error, info, instrument, span, warn, Level};
use std::time::Duration;

#[instrument]
fn fetch_user(user_id: u64) -> Result<String, &'static str> {
    info!(user_id, "Fetching user from database");
    
    // Simulate some work
    std::thread::sleep(Duration::from_millis(100));
    
    if user_id == 404 {
        error!(user_id, "User not found");
        Err("User not found")
    } else {
        let username = format!("user_{}", user_id);
        info!(user_id, username, "User fetched successfully");
        Ok(username)
    }
}

#[instrument]
fn process_request(request_id: String, user_id: u64) {
    let span = span!(Level::INFO, "request_processing", request_id = %request_id);
    let _guard = span.enter();
    
    info!("Processing request");
    
    match fetch_user(user_id) {
        Ok(username) => {
            info!(username, "Request processed successfully");
        }
        Err(e) => {
            warn!(error = e, "Request failed");
        }
    }
    
    debug!("Request processing complete");
}

fn main() {
    // This is the "just works" experience - one line setup!
    better_tracing::fmt().init();
    
    println!("=== Better Tracing Basic Example ===\n");
    
    // Example 1: Simple logging
    info!("Application starting up");
    
    // Example 2: Structured fields
    info!(
        version = env!("CARGO_PKG_VERSION"),
        build_time = "2024-08-02T10:00:00Z",
        "Application initialized"
    );
    
    // Example 3: Spans with context
    let span = span!(Level::INFO, "main_operation", operation_id = "op_123");
    let _guard = span.enter();
    
    // Example 4: Nested operations  
    process_request("req_456".to_string(), 123);
    process_request("req_789".to_string(), 404);
    
    // Example 5: Different log levels
    debug!("Debug information");
    info!("Information message");
    warn!("Warning message");
    error!("Error message");
    
    info!("Application shutting down");
}
