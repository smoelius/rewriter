mod backup;
pub use backup::Backup;

pub mod interface;

mod offset_based_rewriter;

mod offset_calculator;
pub use offset_calculator::OffsetCalculator;

mod rewriter;
pub use rewriter::Rewriter;

mod span;
pub use span::{LineColumn, Span};

#[cfg(feature = "proc-macro2-impl")]
mod proc_macro2_impl;

#[cfg(not(feature = "proc-macro2-span"))]
type SpanDefault = crate::Span;

#[cfg(feature = "proc-macro2-span")]
type SpanDefault = proc_macro2::Span;
