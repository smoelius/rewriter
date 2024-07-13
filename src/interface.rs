use std::fmt::Debug;

pub trait Span: Clone + Debug {
    type LineColumn: self::LineColumn;
    fn line_column(line: usize, column: usize) -> Self::LineColumn;
    fn start(&self) -> Self::LineColumn;
    fn end(&self) -> Self::LineColumn;
}

pub trait LineColumn: Copy + Debug + Ord {
    fn line(&self) -> usize;
    fn line_mut(&mut self) -> &mut usize;
    fn column(&self) -> usize;
    fn column_mut(&mut self) -> &mut usize;
}
