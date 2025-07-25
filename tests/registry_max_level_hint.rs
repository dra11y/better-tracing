#![cfg(all(feature = "registry", feature = "fmt"))]
use better_subscriber::{filter::LevelFilter, prelude::*};

#[test]
fn registry_sets_max_level_hint() {
    better_subscriber::registry()
        .with(better_subscriber::fmt::layer())
        .with(LevelFilter::DEBUG)
        .init();
    assert_eq!(LevelFilter::current(), LevelFilter::DEBUG);
}
