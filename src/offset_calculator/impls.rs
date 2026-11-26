use super::{Interface, OffsetCalculatorError};
use crate::interface::{LineColumn, Span};
use crate::span::LineColumnExt;
use std::{
    iter::Peekable,
    marker::PhantomData,
    str::{Chars, Split},
};

#[derive(Debug)]
pub struct CachingOffsetCalculator<'original, S: Span> {
    line_history: Option<Vec<(usize, bool, &'original str)>>,
    lines: Peekable<Split<'original, char>>,
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
            lines: original.split('\n').peekable(),
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

impl<S: Span> Interface<S> for CachingOffsetCalculator<'_, S> {
    fn offset_from_line_column(
        &mut self,
        line_column: S::LineColumn,
    ) -> Result<(usize, bool), OffsetCalculatorError> {
        if line_column.line() == 0 {
            return Err(OffsetCalculatorError::OutOfBoundsLineColumn);
        }
        if line_column < self.line_column {
            let Some(lines_prev) = &self.line_history else {
                return Err(OffsetCalculatorError::CallWithPastLineColumn {
                    expected_min_line_column: self.line_column.to_crate_line_column(),
                    actual_line_column: line_column.to_crate_line_column(),
                });
            };

            let (line_offset, line_ascii, line) = lines_prev[line_column.line() - 1];

            // smoelius: It is okay to call `str::chars` here because the line is in the past.
            #[allow(clippy::disallowed_methods)]
            let (column_offset, column_ascii) =
                advance_chars(&mut line.chars(), line_column.column())?;

            return Ok((line_offset + column_offset, line_ascii && column_ascii));
        }

        self.advance_to_line(line_column.line());

        let n_columns = line_column.column() - self.line_column.column();

        if let Some(chars) = self.chars_mut() {
            let (offset, ascii) = advance_chars(chars, n_columns)?;
            self.offset += offset;
            self.ascii &= ascii;

            *self.line_column.column_mut() = line_column.column();
        } else {
            return Err(OffsetCalculatorError::OutOfBoundsLineColumn);
        }

        Ok((self.offset, self.ascii))
    }
}

impl<'original, S: Span> CachingOffsetCalculator<'original, S> {
    fn advance_to_line(&mut self, line: usize) {
        if line <= self.line_column.line() {
            return;
        }

        // smoelius: Account for any remaining characters in the current line.
        let Some(suffix) = self.chars_mut().map(|chars| chars.as_str()) else {
            // smoelius: If there are no more characters to read, return.
            return;
        };

        // smoelius: Ensure `chars` is refilled the next time `chars_mut` is called.
        self.chars = None;

        // Count a newline only when another line follows in `self.lines`.
        self.offset += suffix.len() + usize::from(self.lines.peek().is_some());
        self.ascii &= suffix.is_ascii();
        *self.line_column.line_mut() += 1;
        *self.line_column.column_mut() = 0;

        while self.line_column.line() < line {
            let Some(line) = self.next_line() else {
                // smoelius: If there are no more lines to read, return.
                return;
            };

            self.offset += line.len() + usize::from(self.lines.peek().is_some());
            self.ascii &= line.is_ascii();
            *self.line_column.line_mut() += 1;
            *self.line_column.column_mut() = 0;
        }
    }

    /// Returns the contents of [`Self::chars`]
    ///
    /// Calls [`Self::next_line`] if [`Self::chars`] is `None`. Returns `None` if there are no more
    /// lines to read.
    fn chars_mut(&mut self) -> Option<&mut Chars<'original>> {
        #[allow(clippy::disallowed_methods)]
        if self.chars.is_none() {
            let line = self.next_line()?;
            self.chars = Some(line.chars());
        }
        self.chars.as_mut()
    }

    /// Fetches the next line from [`Self::lines`]
    ///
    /// Returns `None` if there are no more lines to read.
    fn next_line(&mut self) -> Option<&'original str> {
        let line = self.lines.next()?;
        if let Some(line_history) = &mut self.line_history {
            line_history.push((self.offset, self.ascii, line));
        }
        Some(line)
    }
}

#[allow(clippy::disallowed_methods)]
impl<S: Span> Interface<S> for StatelessOffsetCalculator<'_, S> {
    #[cfg_attr(
        dylint_lib = "misleading_variable_name",
        allow(misleading_variable_name)
    )]
    fn offset_from_line_column(
        &mut self,
        line_column: S::LineColumn,
    ) -> Result<(usize, bool), OffsetCalculatorError> {
        if line_column.line() == 0 {
            return Err(OffsetCalculatorError::OutOfBoundsLineColumn);
        }
        let mut lines = self.original.split('\n').peekable();
        let mut offset = 0;
        let mut ascii = true;

        for _ in 1..line_column.line() {
            let line = lines
                .next()
                .ok_or(OffsetCalculatorError::OutOfBoundsLineColumn)?;
            // Like the caching calculator, count a newline only when another line follows in
            // `lines`.
            offset += line.len() + usize::from(lines.peek().is_some());
            ascii &= line.is_ascii();
        }

        let line = lines
            .next()
            .ok_or(OffsetCalculatorError::OutOfBoundsLineColumn)?;
        let (column_offset, column_ascii) = advance_chars(&mut line.chars(), line_column.column())?;
        offset += column_offset;
        ascii &= column_ascii;

        Ok((offset, ascii))
    }
}

fn advance_chars(chars: &mut Chars, n: usize) -> Result<(usize, bool), OffsetCalculatorError> {
    let mut remaining = chars.clone();
    let mut offset = 0;
    let mut ascii = true;

    for _ in 0..n {
        let ch = remaining
            .next()
            .ok_or(OffsetCalculatorError::OutOfBoundsLineColumn)?;
        offset += ch.len_utf8();
        ascii &= ch.is_ascii();
    }

    *chars = remaining;
    Ok((offset, ascii))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LineColumn, Span};

    fn out_of_bounds_cases()
    -> impl Iterator<Item = (&'static str, LineColumn, Option<(usize, bool)>)> {
        const ORIGINALS: &[&str] = &[
            "", "\n", "\n\n", "x", "x\n", "x\ny", "x\ny\n", "χ", "χ\n", "χ\nψ", "χ\nψ\n",
        ];
        let originals_with_last_lines = ORIGINALS.iter().map(|&original| {
            let (index, line) = original.split('\n').enumerate().last().unwrap();
            (original, index, line)
        });
        originals_with_last_lines.flat_map(|(original, index, line)| {
            #[allow(clippy::disallowed_methods)]
            let end_column = line.chars().count();
            [end_column, end_column + 1, usize::MAX]
                .into_iter()
                .map(move |column| {
                    (
                        original,
                        LineColumn {
                            line: index + 1,
                            column,
                        },
                        (column == end_column).then_some((original.len(), original.is_ascii())),
                    )
                })
                .chain([0, 1, usize::MAX].into_iter().map(move |column| {
                    (
                        original,
                        LineColumn {
                            line: index + 2,
                            column,
                        },
                        None,
                    )
                }))
                .chain([0, 1, usize::MAX].into_iter().map(move |column| {
                    (
                        original,
                        LineColumn {
                            line: index + 3,
                            column,
                        },
                        None,
                    )
                }))
        })
    }

    #[test]
    fn out_of_bounds_lookup() {
        for (original, line_column, expected) in out_of_bounds_cases() {
            let mut stateless = StatelessOffsetCalculator::<Span>::new(original);

            assert_eq!(
                expected,
                stateless.offset_from_line_column(line_column).ok(),
                "original = {original:?}, line_column = {line_column:?}",
            );

            for mut caching in [
                CachingOffsetCalculator::<Span>::new(original),
                CachingOffsetCalculator::<Span>::without_line_history(original),
            ] {
                assert_eq!(
                    expected,
                    caching.offset_from_line_column(line_column).ok(),
                    "original = {original:?}, line_column = {line_column:?}",
                );
                // Repeated lookups must preserve the exhausted state.
                assert_eq!(
                    expected,
                    caching.offset_from_line_column(line_column).ok(),
                    "repeated lookup: original = {original:?}, line_column = {line_column:?}",
                );
            }
        }
    }

    #[test]
    fn out_of_bounds_lookup_after_reading_all_characters() {
        for (original, line_column, expected) in out_of_bounds_cases() {
            let mut stateless = StatelessOffsetCalculator::<Span>::new(original);
            for mut caching in [
                CachingOffsetCalculator::<Span>::new(original),
                CachingOffsetCalculator::<Span>::without_line_history(original),
            ] {
                for (index, line) in original.split('\n').enumerate() {
                    #[allow(clippy::disallowed_methods)]
                    for column in 0..=line.chars().count() {
                        let line_column = LineColumn {
                            line: index + 1,
                            column,
                        };
                        assert_eq!(
                            stateless.offset_from_line_column(line_column).unwrap(),
                            caching.offset_from_line_column(line_column).unwrap(),
                            "original = {original:?}, line_column = {line_column:?}",
                        );
                    }
                }
                assert_eq!(
                    expected,
                    caching.offset_from_line_column(line_column).ok(),
                    "original = {original:?}, line_column = {line_column:?}",
                );
            }
        }
    }

    #[test]
    fn out_of_bounds_lookup_followed_by_past_line_lookups() {
        for (original, line_column, expected) in out_of_bounds_cases() {
            let mut stateless = StatelessOffsetCalculator::<Span>::new(original);
            // smoelius: A `CachingOffsetCalculator` constructed with `without_line_history` does
            // not support past line lookups.
            let mut caching = CachingOffsetCalculator::<Span>::new(original);
            assert_eq!(
                expected,
                caching.offset_from_line_column(line_column).ok(),
                "original = {original:?}, line_column = {line_column:?}",
            );
            let line = original.split('\n').count();
            for index in (0..line).rev() {
                let line_column = LineColumn {
                    line: index + 1,
                    column: 0,
                };
                assert_eq!(
                    stateless.offset_from_line_column(line_column).unwrap(),
                    caching.offset_from_line_column(line_column).unwrap(),
                    "original = {original:?}, line_column = {line_column:?}",
                );
            }
        }
    }
}
