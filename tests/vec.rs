#![cfg(feature = "registry")]
use better_tracing::prelude::*;
use tracing::level_filters::LevelFilter;
use tracing::Subscriber;

#[test]
fn just_empty_vec() {
    // Just a None means everything is off
    let subscriber = better_tracing::registry().with(Vec::<LevelFilter>::new());
    assert_eq!(subscriber.max_level_hint(), Some(LevelFilter::OFF));
}

#[test]
fn layer_and_empty_vec() {
    let subscriber = better_tracing::registry()
        .with(LevelFilter::INFO)
        .with(Vec::<LevelFilter>::new());
    assert_eq!(subscriber.max_level_hint(), Some(LevelFilter::INFO));
}
