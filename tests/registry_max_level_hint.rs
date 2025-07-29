#![cfg(all(feature = "registry", feature = "fmt"))]
use better_tracing::{filter::LevelFilter, prelude::*};

#[test]
fn registry_sets_max_level_hint() {
    better_tracing::registry()
        .with(better_tracing::fmt::layer())
        .with(LevelFilter::DEBUG)
        .init();
    assert_eq!(LevelFilter::current(), LevelFilter::DEBUG);
}
