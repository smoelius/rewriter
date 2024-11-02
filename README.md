# rewriter

Rust utilities for rewriting files

## Main exports

- [`Rewriter`]: rewrites a file's contents

- [`Backup`]: restores a file's contents when dropped, unless explicitly disabled

- [`Span`]: names a region of a file, similar to [`proc-macro2::Span`]

- [`LineColumn`]: names a point in a file, similar to [`proc-macro2::LineColumn`]

- [`interface::Span`](https://docs.rs/rewriter/latest/rewriter/interface/trait.Span.html) and [`interface::LineColumn`](https://docs.rs/rewriter/latest/rewriter/interface/trait.LineColumn.html): traits that span and line-column types must implement to be used with this library

## Features

- `proc-macro2-span`: By default `rewriter`, uses its own `Span` and `LineColumn` types. When this feature is enabled, `rewriter` instead uses the corresponding types from [`proc-macro2`]. This feature implies `proc-macro2-impl` below.

- `proc-macro2-impl`: Implement the `Span` and `LineColumn` traits for the corresponding types in [`proc-macro2`].

[`Backup`]: https://docs.rs/rewriter/latest/rewriter/struct.Backup.html
[`LineColumn`]: https://docs.rs/rewriter/latest/rewriter/struct.LineColumn.html
[`Rewriter`]: https://docs.rs/rewriter/latest/rewriter/struct.Rewriter.html
[`Span`]: https://docs.rs/rewriter/latest/rewriter/struct.Span.html
[`proc-macro2::LineColumn`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.LineColumn.html
[`proc-macro2::Span`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.Span.html
[`proc-macro2`]: https://crates.io/crates/proc-macro2
