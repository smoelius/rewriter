use crate::{LineColumn, SpanDefault, interface::Span};
use thiserror::Error;

#[cfg(feature = "__check-proc-macro2-spans")]
use std::sync::atomic::{AtomicUsize, Ordering};

mod impls;

use impls::CachingOffsetCalculator;

#[cfg(feature = "check-offsets")]
use impls::StatelessOffsetCalculator;

#[cfg(feature = "__check-proc-macro2-spans")]
static BASE_NEXT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum OffsetCalculatorError {
    #[error(
        "`offset_from_line_column` called with a `LineColumn` in the past: {actual_line_column:?} \
         < {expected_min_line_column:?}"
    )]
    CallWithPastLineColumn {
        expected_min_line_column: LineColumn,
        actual_line_column: LineColumn,
    },
    #[error("span end is before span start")]
    InvalidSpan,
    #[error("line or column is out of bounds")]
    OutOfBoundsLineColumn,
}

pub trait Interface<S: Span> {
    /// Returns the byte offset for `line_column`
    ///
    /// The second component (the `bool`) indicates whether all characters up to the offset are
    /// ASCII.
    fn offset_from_line_column(
        &mut self,
        line_column: S::LineColumn,
    ) -> Result<(usize, bool), OffsetCalculatorError>;
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
            base: BASE_NEXT.fetch_add(1 + original.len(), Ordering::SeqCst),
        }
    }

    pub fn offsets_from_span(&mut self, span: &S) -> Result<(usize, usize), OffsetCalculatorError> {
        if span.start() > span.end() {
            return Err(OffsetCalculatorError::InvalidSpan);
        }

        let (start, start_ascii) = self.offset_from_line_column(span.start())?;
        let (end, end_ascii) = self.offset_from_line_column(span.end())?;

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

        Ok((start, end))
    }
}

impl<S: Span> Interface<S> for OffsetCalculator<'_, S> {
    fn offset_from_line_column(
        &mut self,
        line_column: S::LineColumn,
    ) -> Result<(usize, bool), OffsetCalculatorError> {
        let result = self.caching.offset_from_line_column(line_column);

        #[cfg(feature = "check-offsets")]
        if !matches!(
            &result,
            Err(OffsetCalculatorError::CallWithPastLineColumn { .. })
        ) {
            assert_eq!(
                result.as_ref().ok().copied(),
                self.stateless.offset_from_line_column(line_column).ok(),
                "failed for {line_column:?}"
            );
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[expect(dead_code, clippy::unnecessary_map_on_constructor)]
    fn offset_calculator_error_converts_to_anyhow() -> Result<()> {
        Err(OffsetCalculatorError::CallWithPastLineColumn {
            expected_min_line_column: LineColumn::default(),
            actual_line_column: LineColumn::default(),
        })
        .map_err(Into::into)
    }
}
