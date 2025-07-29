//! The `better-tracing` prelude.
//!
//! This brings into scope a number of extension traits that define methods on
//! types defined here and in other crates.

pub use crate::field::{
    MakeExt as __better_tracing_field_MakeExt, RecordFields as __better_tracing_field_RecordFields,
};
pub use crate::layer::{
    Layer as __better_tracing_Layer, SubscriberExt as __better_tracing_SubscriberExt,
};

pub use crate::util::SubscriberInitExt as _;

feature! {
    #![all(feature = "fmt", feature = "std")]
    pub use crate::fmt::writer::MakeWriterExt as _;
}
