use crate::interface::Span as _;
use crate::offset_based_rewriter::{self, OffsetBasedRewriter};
use crate::offset_calculator::OffsetCalculator;
use crate::span::{LineColumn, Span};

#[derive(Debug)]
pub struct Rewriter<'original> {
    line_column: LineColumn,
    offset_calculator: OffsetCalculator<'original>,
    offset_based_rewriter: OffsetBasedRewriter<'original>,
}

impl<'original> Rewriter<'original> {
    pub fn new(original: &'original str) -> Self {
        Self {
            line_column: LineColumn { line: 1, column: 0 },
            offset_calculator: OffsetCalculator::new(original),
            offset_based_rewriter: OffsetBasedRewriter::new(original),
        }
    }

    pub fn contents(self) -> String {
        use offset_based_rewriter::Interface;

        self.offset_based_rewriter.contents()
    }

    pub fn rewrite(&mut self, span: Span, replacement: &str) -> String {
        use offset_based_rewriter::Interface;

        assert!(
            self.line_column <= span.start(),
            "self = {:#?}, span.start() = {:?}, span.end() = {:?}",
            self,
            span.start(),
            span.end(),
        );

        let (start, end) = self.offset_calculator.offsets_from_span(span);

        let replaced = self.offset_based_rewriter.rewrite(start, end, replacement);

        self.line_column = span.end();

        replaced
    }
}
