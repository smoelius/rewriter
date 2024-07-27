# rewriter

Rust utilities for rewriting files

## Main exports

- `Rewriter` type: for rewriting the contents of files

- `Backup` type: restore a file's contents when dropped, unless explicitly disabled

- `Span` type: names a region of a file, similar to [`proc-macro2::Span`]

- `LineColumn` type: names a point in a file, similar to [`proc-macro2::LineColumn`]

- `Span` and `LineColumn` traits: allow span and line-column types to be used with this library

## Features

- `proc-macro2-span`: By default `rewriter`, uses its own `Span` and `LineColumn` types. When this feature is enabled, `rewriter` instead uses the corresponding types from [`proc-macro2`]. This feature implies `proc-macro2-impl` below.

- `proc-macro2-impl`: Implement the `Span` and `LineColumn` traits for the corresponding types in [`proc-macro2`].

[`proc-macro2::LineColumn`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.LineColumn.html
[`proc-macro2::Span`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.Span.html
[`proc-macro2`]: https://crates.io/crates/proc-macro2
