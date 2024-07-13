use crate::interface::Span as _;
use crate::span::{LineColumn, Span};

#[cfg(feature = "__check-proc-macro2-spans")]
use std::sync::atomic::{AtomicUsize, Ordering};

mod impls;

use impls::CachingOffsetCalculator;

#[cfg(feature = "check-offsets")]
use impls::StatelessOffsetCalculator;

#[cfg(feature = "__check-proc-macro2-spans")]
static BASE_NEXT: AtomicUsize = AtomicUsize::new(0);

pub trait Interface {
    fn offset_from_line_column(&mut self, line_column: LineColumn) -> (usize, bool);
}

#[derive(Debug)]
pub struct OffsetCalculator<'original> {
    caching: CachingOffsetCalculator<'original>,

    #[cfg(feature = "check-offsets")]
    stateless: StatelessOffsetCalculator<'original>,

    #[cfg(feature = "__check-proc-macro2-spans")]
    base: usize,
}

impl<'original> OffsetCalculator<'original> {
    pub fn new(original: &'original str) -> Self {
        #[cfg(feature = "__check-proc-macro2-spans")]
        let base = BASE_NEXT.fetch_add(1 + original.as_bytes().len(), Ordering::SeqCst);

        Self {
            caching: CachingOffsetCalculator::new(original),

            #[cfg(feature = "check-offsets")]
            stateless: StatelessOffsetCalculator::new(original),

            #[cfg(feature = "__check-proc-macro2-spans")]
            base,
        }
    }

    pub fn offsets_from_span(&mut self, span: Span) -> (usize, usize) {
        let (start, start_ascii) = self.offset_from_line_column(span.start());
        let (end, end_ascii) = self.offset_from_line_column(span.end());

        assert!(!end_ascii || start_ascii);

        // smoelius: `proc_macro2::Span`'s debug output doesn't seem to account for UTF-8.
        #[cfg(feature = "__check-proc-macro2-spans")]
        if end_ascii {
            let start = self.base + start;
            let end = self.base + end;
            assert_eq!(
                format!("{span:?}"),
                format!("bytes({}..{})", 1 + start, 1 + end),
                "self = {:#?}, span.start() = {:?}, span.end() = {:?}",
                self,
                span.start(),
                span.end(),
            );
        }

        (start, end)
    }
}

impl<'original> Interface for OffsetCalculator<'original> {
    fn offset_from_line_column(&mut self, line_column: LineColumn) -> (usize, bool) {
        let (offset, ascii) = self.caching.offset_from_line_column(line_column);

        #[cfg(feature = "check-offsets")]
        {
            let (offset_comparator, ascii_comparator) =
                self.stateless.offset_from_line_column(line_column);
            assert_eq!(offset, offset_comparator);
            assert_eq!(ascii, ascii_comparator);
        }

        (offset, ascii)
    }
}
