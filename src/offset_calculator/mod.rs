use crate::interface::Span;
use crate::SpanDefault;

#[cfg(feature = "__check-proc-macro2-spans")]
use std::sync::atomic::{AtomicUsize, Ordering};

mod impls;

use impls::CachingOffsetCalculator;

#[cfg(feature = "check-offsets")]
use impls::StatelessOffsetCalculator;

#[cfg(feature = "__check-proc-macro2-spans")]
static BASE_NEXT: AtomicUsize = AtomicUsize::new(0);

pub trait Interface<S: Span> {
    /// Returns the byte offset for `line_column`
    ///
    /// The second component (the `bool`) indicates whether all characters up to the offset are
    /// ASCII.
    fn offset_from_line_column(&mut self, line_column: S::LineColumn) -> (usize, bool);
}

#[derive(Debug)]
pub struct OffsetCalculator<'original, S: Span = SpanDefault> {
    caching: CachingOffsetCalculator<'original, S>,

    #[cfg(feature = "check-offsets")]
    stateless: StatelessOffsetCalculator<'original, S>,

    #[cfg(feature = "__check-proc-macro2-spans")]
    base: usize,
}

impl<'original, S: Span> OffsetCalculator<'original, S> {
    #[must_use]
    pub fn new(original: &'original str) -> Self {
        Self::new_private(original, true)
    }

    pub(crate) fn new_private(original: &'original str, with_line_history: bool) -> Self {
        let caching = if with_line_history {
            CachingOffsetCalculator::new(original)
        } else {
            CachingOffsetCalculator::without_line_history(original)
        };

        Self {
            caching,

            #[cfg(feature = "check-offsets")]
            stateless: StatelessOffsetCalculator::new(original),

            #[cfg(feature = "__check-proc-macro2-spans")]
            base: BASE_NEXT.fetch_add(1 + original.as_bytes().len(), Ordering::SeqCst),
        }
    }

    pub fn offsets_from_span(&mut self, span: &S) -> (usize, usize) {
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

impl<S: Span> Interface<S> for OffsetCalculator<'_, S> {
    fn offset_from_line_column(&mut self, line_column: S::LineColumn) -> (usize, bool) {
        let (offset, ascii) = self.caching.offset_from_line_column(line_column);

        #[cfg(feature = "check-offsets")]
        {
            let (offset_comparator, ascii_comparator) =
                self.stateless.offset_from_line_column(line_column);
            assert_eq!(offset, offset_comparator, "failed for {line_column:?}");
            assert_eq!(ascii, ascii_comparator, "failed for {line_column:?}");
        }

        (offset, ascii)
    }
}
