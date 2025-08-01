#![cfg(feature = "fmt")]
use better_tracing::filter::LevelFilter;

#[test]
fn fmt_sets_max_level_hint() {
    better_tracing::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();
    assert_eq!(LevelFilter::current(), LevelFilter::DEBUG);
}
