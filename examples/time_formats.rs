use better_tracing::fmt::time::SystemTime;

fn main() {
    // Default (RFC3339 micros, date+time)
    let _ = better_tracing::fmt().with_timer(SystemTime).try_init();

    tracing::info!("default timestamp");

    // RFC3339 variants
    let _ = better_tracing::fmt()
        .with_timer(SystemTime::rfc3339_seconds())
        .try_init();
    tracing::info!("rfc3339 seconds");

    let _ = better_tracing::fmt()
        .with_timer(SystemTime::rfc3339_millis())
        .try_init();
    tracing::info!("rfc3339 millis");

    let _ = better_tracing::fmt()
        .with_timer(SystemTime::rfc3339_nanos())
        .try_init();
    tracing::info!("rfc3339 nanos");

    // Unix epoch variants
    let _ = better_tracing::fmt()
        .with_timer(SystemTime::unix_seconds())
        .try_init();
    tracing::info!("unix seconds");

    let _ = better_tracing::fmt()
        .with_timer(SystemTime::unix_millis())
        .try_init();
    tracing::info!("unix millis");

    // Time-only variants (no date)
    let _ = better_tracing::fmt()
        .with_timer(SystemTime::time_only_secs())
        .try_init();
    tracing::info!("time only sec");

    let _ = better_tracing::fmt()
        .with_timer(SystemTime::time_only_millis())
        .try_init();
    tracing::info!("time only ms");
}
