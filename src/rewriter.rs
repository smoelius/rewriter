use crate::SpanDefault;
use crate::interface::Span;
use crate::offset_based_rewriter::{self, OffsetBasedRewriter};
use crate::offset_calculator::{OffsetCalculator, OffsetCalculatorError};
use std::{cell::RefCell, rc::Rc};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RewriterError<S: Span> {
    #[error("expected_min_line_column = {expected_min_line_column:#?}, actual_span.start() = {:?}, \
             actual_span.end() = {:?}", actual_span.start(), actual_span.end())]
    CallWithPastSpan {
        expected_min_line_column: S::LineColumn,
        actual_span: S,
    },
    #[error(transparent)]
    OffsetCalculatorError(#[from] OffsetCalculatorError<S>),
}

#[derive(Debug)]
pub struct Rewriter<'original, S: Span = SpanDefault> {
    line_column: S::LineColumn,
    offset_calculator: Rc<RefCell<OffsetCalculator<'original, S>>>,
    offset_based_rewriter: OffsetBasedRewriter<'original>,
}

impl<'original, S: Span> Rewriter<'original, S> {
    #[must_use]
    pub fn new(original: &'original str) -> Self {
        let offset_calculator =
            Rc::new(RefCell::new(OffsetCalculator::new_private(original, false)));
        Self::with_offset_calculator(original, offset_calculator)
    }

    pub fn with_offset_calculator(
        original: &'original str,
        offset_calculator: Rc<RefCell<OffsetCalculator<'original, S>>>,
    ) -> Self {
        Self {
            line_column: S::line_column(1, 0),
            offset_calculator,
            offset_based_rewriter: OffsetBasedRewriter::new(original),
        }
    }

    pub fn contents(self) -> String {
        use offset_based_rewriter::Interface;

        self.offset_based_rewriter.contents()
    }

    pub fn rewrite(&mut self, span: &S, replacement: &str) -> Result<String, RewriterError<S>> {
        use offset_based_rewriter::Interface;

        if self.line_column > span.start() {
            return Err(RewriterError::CallWithPastSpan {
                expected_min_line_column: self.line_column,
                actual_span: span.clone(),
            });
        }

        let (start, end) = self
            .offset_calculator
            .borrow_mut()
            .offsets_from_span(span)?;

        let replaced = self.offset_based_rewriter.rewrite(start, end, replacement);

        self.line_column = span.end();

        Ok(replaced)
    }
}
