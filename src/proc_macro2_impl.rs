use crate::interface;

impl interface::LineColumn for proc_macro2::LineColumn {
    fn line(&self) -> usize {
        self.line
    }
    fn line_mut(&mut self) -> &mut usize {
        &mut self.line
    }
    fn column(&self) -> usize {
        self.column
    }
    fn column_mut(&mut self) -> &mut usize {
        &mut self.column
    }
}

impl interface::Span for proc_macro2::Span {
    type LineColumn = proc_macro2::LineColumn;
    fn line_column(line: usize, column: usize) -> Self::LineColumn {
        proc_macro2::LineColumn { line, column }
    }
    fn start(&self) -> Self::LineColumn {
        self.start()
    }
    fn end(&self) -> Self::LineColumn {
        self.end()
    }
}
