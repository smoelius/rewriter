# rewriter

Rust utilities for rewriting files

## Main exports

- [`Rewriter`]: rewrites a file's contents

- [`Backup`]: restores a file's contents when dropped, unless explicitly disabled

- [`Span`]: names a region of a file, similar to [`proc-macro2::Span`]

- [`LineColumn`]: names a point in a file, similar to [`proc-macro2::LineColumn`]

- [`interface::Span`](https://docs.rs/rewriter/latest/rewriter/interface/trait.Span.html) and [`interface::LineColumn`](https://docs.rs/rewriter/latest/rewriter/interface/trait.LineColumn.html): traits that span and line-column types must implement to be used with this library

## Features

- `proc-macro2-span`: By default, `rewriter` uses its own `Span` and `LineColumn` types. When this feature is enabled, `rewriter` instead uses the corresponding types from [`proc-macro2`]. This feature implies `proc-macro2-impl` below.

- `proc-macro2-impl`: Implement the `Span` and `LineColumn` traits for the corresponding types in [`proc-macro2`].

## Comparison to `TextEdit`

The closest analogue to `Rewriter` of which we are aware is [`rust-analyzer`]'s [`TextEdit`].

The following are some notable differences:

- **Interface:** One constructs a `Rewriter` from a `&str`, performs a series of [`rewrite`]s, and then calls [`contents`] to obtain the resulting text as a `String`. By comparison, one constructs a [`TextEdit`] and then [`apply`]s it to an `&mut String`.

- **Referring to text:** `Rewriter` refers to text using `Span`s, which are composed of pairs of `LineColumn`s. Each `LineColumn` contains a line and a column. By comparison, `TextEdit` refers to text using [`TextRange`]s, which are composed of pairs of [`TextSize`]s. Each `TextSize` wraps a `u32` byte offset.

- **Stability:** `Rewriter` aims to provide a stable interface. By comparison, users of the `rust-analyzer` crates are [advised to pin versions or expect regular breaking changes].

[`Backup`]: https://docs.rs/rewriter/latest/rewriter/struct.Backup.html
[`LineColumn`]: https://docs.rs/rewriter/latest/rewriter/struct.LineColumn.html
[`Rewriter`]: https://docs.rs/rewriter/latest/rewriter/struct.Rewriter.html
[`Span`]: https://docs.rs/rewriter/latest/rewriter/struct.Span.html
[`TextEdit`]: https://docs.rs/ra_ap_text_edit/latest/ra_ap_text_edit/struct.TextEdit.html
[`TextRange`]: https://docs.rs/text-size/latest/text_size/struct.TextRange.html
[`TextSize`]: https://docs.rs/text-size/latest/text_size/struct.TextSize.html
[`apply`]: https://docs.rs/ra_ap_text_edit/latest/ra_ap_text_edit/struct.TextEdit.html#method.apply
[`contents`]: https://docs.rs/rewriter/latest/rewriter/struct.Rewriter.html#method.contents
[`proc-macro2::LineColumn`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.LineColumn.html
[`proc-macro2::Span`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.Span.html
[`proc-macro2`]: https://crates.io/crates/proc-macro2
[`rewrite`]: https://docs.rs/rewriter/latest/rewriter/struct.Rewriter.html#method.rewrite
[`rust-analyzer`]: https://rust-analyzer.github.io/
[advised to pin versions or expect regular breaking changes]: https://github.com/rust-lang/rust-analyzer/issues/11615#issuecomment-1059074190
