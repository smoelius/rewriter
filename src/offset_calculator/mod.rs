use crate::span::LineColumn;

mod impls;

use impls::CachingOffsetCalculator;

#[cfg(feature = "check-offsets")]
use impls::StatelessOffsetCalculator;

pub trait Interface {
    fn offset_from_line_column(&mut self, line_column: LineColumn) -> (usize, bool);
}

#[derive(Debug)]
pub struct OffsetCalculator<'original> {
    caching: CachingOffsetCalculator<'original>,

    #[cfg(feature = "check-offsets")]
    stateless: StatelessOffsetCalculator<'original>,
}

impl<'original> OffsetCalculator<'original> {
    pub fn new(original: &'original str) -> Self {
        Self {
            caching: CachingOffsetCalculator::new(original),

            #[cfg(feature = "check-offsets")]
            stateless: StatelessOffsetCalculator::new(original),
        }
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
