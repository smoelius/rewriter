use super::Interface;
use crate::interface::{LineColumn, Span};
use std::{
    marker::PhantomData,
    str::{Chars, Split},
};

#[derive(Debug)]
pub struct CachingOffsetCalculator<'original, S: Span> {
    line_history: Option<Vec<(usize, bool, &'original str)>>,
    lines: Split<'original, char>,
    chars: Option<Chars<'original>>,
    line_column: S::LineColumn,
    offset: usize,
    ascii: bool,
}

#[derive(Debug)]
pub struct StatelessOffsetCalculator<'original, S: Span> {
    original: &'original str,
    phantom: PhantomData<S>,
}

impl<'original, S: Span> CachingOffsetCalculator<'original, S> {
    pub fn new(original: &'original str) -> Self {
        let mut caching_offset_calculator = Self::without_line_history(original);
        caching_offset_calculator.line_history = Some(Vec::new());
        caching_offset_calculator
    }

    pub fn without_line_history(original: &'original str) -> Self {
        Self {
            line_history: None,
            lines: original.split('\n'),
            chars: None,
            line_column: S::line_column(1, 0),
            offset: 0,
            ascii: true,
        }
    }
}

impl<'original, S: Span> StatelessOffsetCalculator<'original, S> {
    #[allow(dead_code)]
    pub fn new(original: &'original str) -> Self {
        Self {
            original,
            phantom: PhantomData,
        }
    }
}

impl<'original, S: Span> Interface<S> for CachingOffsetCalculator<'original, S> {
    fn offset_from_line_column(&mut self, line_column: S::LineColumn) -> (usize, bool) {
        if line_column < self.line_column {
            let Some(lines_prev) = &self.line_history else {
                panic!(
                    "`offset_from_line_column` called on a `LineColumn` in the past: {:?} < {:?}",
                    line_column, self.line_column
                )
            };

            let (line_offset, line_ascii, line) = lines_prev[line_column.line() - 1];

            // smoelius: It is okay to call `str::chars` here because the line is in the past.
            #[allow(clippy::disallowed_methods)]
            let (column_offset, column_ascii) =
                advance_chars(&mut line.chars(), line_column.column());

            return (line_offset + column_offset, line_ascii && column_ascii);
        }

        self.advance_to_line(line_column.line());

        let n_columns = line_column.column() - self.line_column.column();

        let chars = self.chars_mut();

        let (offset, ascii) = advance_chars(chars, n_columns);
        self.offset += offset;
        self.ascii &= ascii;

        *self.line_column.column_mut() = line_column.column();

        (self.offset, self.ascii)
    }
}

impl<'original, S: Span> CachingOffsetCalculator<'original, S> {
    fn advance_to_line(&mut self, line: usize) {
        if line <= self.line_column.line() {
            return;
        }

        // smoelius: Account for any remaining characters in the current line.
        let suffix = self.chars_mut().collect::<String>();

        // smoelius: Ensure `chars` is refilled the next time `chars_mut` is called.
        self.chars = None;

        self.offset += suffix.as_bytes().len() + 1;
        self.ascii &= suffix.is_ascii();
        *self.line_column.line_mut() += 1;
        *self.line_column.column_mut() = 0;

        while self.line_column.line() < line {
            let line = self.next_line();

            self.offset += line.as_bytes().len() + 1;
            self.ascii &= line.is_ascii();
            *self.line_column.line_mut() += 1;
            *self.line_column.column_mut() = 0;
        }
    }

    /// Returns the contents of [`Self::chars`]. Calls [`Self::next_line`] if [`Self::chars`] is
    /// `None`.
    fn chars_mut(&mut self) -> &mut Chars<'original> {
        #[allow(clippy::disallowed_methods)]
        if self.chars.is_none() {
            self.chars = Some(self.next_line().chars());
        }
        self.chars.as_mut().unwrap()
    }

    /// Fetches the next line from [`Self::lines`]
    fn next_line(&mut self) -> &'original str {
        let line = self.lines.next().unwrap();
        if let Some(line_history) = &mut self.line_history {
            line_history.push((self.offset, self.ascii, line));
        }
        line
    }
}

fn advance_chars(chars: &mut Chars, n: usize) -> (usize, bool) {
    let prefix = chars.take(n).collect::<String>();
    let offset = prefix.as_bytes().len();
    let ascii = prefix.is_ascii();
    (offset, ascii)
}

#[allow(clippy::disallowed_methods)]
impl<'original, S: Span> Interface<S> for StatelessOffsetCalculator<'original, S> {
    #[cfg_attr(
        dylint_lib = "misleading_variable_name",
        allow(misleading_variable_name)
    )]
    fn offset_from_line_column(&mut self, line_column: S::LineColumn) -> (usize, bool) {
        let mut lines = self.original.split('\n');
        let mut offset = 0;
        let mut ascii = true;

        for _ in 1..line_column.line() {
            let line = lines.next().unwrap();
            offset += line.as_bytes().len() + 1;
            ascii &= line.is_ascii();
        }

        let prefix = lines
            .next()
            .unwrap()
            .chars()
            .take(line_column.column())
            .collect::<String>();
        offset += prefix.as_bytes().len();
        ascii &= prefix.is_ascii();

        (offset, ascii)
    }
}
