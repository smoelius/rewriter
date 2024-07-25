use crate::interface::Span;
use crate::offset_based_rewriter::{self, OffsetBasedRewriter};
use crate::offset_calculator::OffsetCalculator;
use crate::SpanDefault;
use std::{cell::RefCell, rc::Rc};

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

    pub fn rewrite(&mut self, span: &S, replacement: &str) -> String {
        use offset_based_rewriter::Interface;

        assert!(
            self.line_column <= span.start(),
            "self = {:#?}, span.start() = {:?}, span.end() = {:?}",
            self,
            span.start(),
            span.end(),
        );

        let (start, end) = self.offset_calculator.borrow_mut().offsets_from_span(span);

        let replaced = self.offset_based_rewriter.rewrite(start, end, replacement);

        self.line_column = span.end();

        replaced
    }
}
