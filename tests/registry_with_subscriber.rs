#![cfg(feature = "registry")]
use tracing_futures::{Instrument, WithSubscriber};
use better_subscriber::prelude::*;

#[tokio::test]
async fn future_with_subscriber() {
    better_subscriber::registry().init();
    let span = tracing::info_span!("foo");
    let _e = span.enter();
    let span = tracing::info_span!("bar");
    let _e = span.enter();
    tokio::spawn(
        async {
            async {
                let span = tracing::Span::current();
                println!("{:?}", span);
            }
            .instrument(tracing::info_span!("hi"))
            .await
        }
        .with_subscriber(better_subscriber::registry()),
    )
    .await
    .unwrap();
}
