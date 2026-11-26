# Changelog

## 3.0.0

- Avoid temporary strings in offset calculations ([536173a](https://github.com/smoelius/rewriter/commit/536173a6f5eb1ddff08232741e543c57860a3a3b))
- BREAKING: `Rewriter::rewrite` and `OffsetCalculator::offsets_from_span` now return `Result` ([f42d20e](https://github.com/smoelius/rewriter/commit/f42d20e7fd12d4b07f78feab4c9234dcf7b3b6c5), [b0c9c73](https://github.com/smoelius/rewriter/commit/b0c9c73eec8ea2d8bea9dda7d90620f6068b18f1), [0521484](https://github.com/smoelius/rewriter/commit/052148401cbc790eb92c8c68b5acd437dcd44f59), and [04da99b](https://github.com/smoelius/rewriter/commit/04da99b03d6ddf4d6c276411642e521dd6897f53))
- Fix insertion at end of input ([e2cc2ad](https://github.com/smoelius/rewriter/commit/e2cc2ad5e48112381e8d3049c3e5d8ee67bf09bc))

## 2.1.1

- Fix offset calculation beyond the end of input ([2a97a35](https://github.com/smoelius/rewriter/commit/2a97a359044558325d3fa96fc08130d8643798a9))

## 2.1.0

- FEATURE: Add `Backup::path` method to return the path to the `Backup`'s original file ([c0c02c5](https://github.com/smoelius/rewriter/commit/c0c02c58b93be615e66aa0a80fa26bf29c1cc063))

## 2.0.0

- BREAKING: When `Backup` is dropped, read the original file's permissions and make a best-effort attempt to preserve them. Previously, permissions changes made while the `Backup` was alive were discarded. This fixes a bug in version 1.0.0 where `FILE_ATTRIBUTE_TEMPORARY` could be set on the original file on Windows. We say "best-effort" because certain permissions changes could cause the restoration to fail. For example, if the original file were to become read-only, the restoration would fail. Note that `Backup::new` still requires the original file to be writable when the function is called. ([8f81d3a](https://github.com/smoelius/rewriter/commit/8f81d3a105199aa34b2337143171049d9023fd0c))

## 1.0.0

- Add `TextEdit` comparison to README.md ([5fd728b](https://github.com/smoelius/rewriter/commit/5fd728be7fdca8309aa41216ff98dc9990010b5f))
- Set file `mtime` explicitly rather than manually copying file contents ([e20c64e](https://github.com/smoelius/rewriter/commit/e20c64edafe35993a28dff22feacfa28e14c65af))
- BREAKING: Make backup files read-only ([f2ecaee](https://github.com/smoelius/rewriter/commit/f2ecaeebdf9cab338b76ec0e3b2664b67961f30a))
- BREAKING: Document `Backup::new`, including requirement that original file be writable ([c166010](https://github.com/smoelius/rewriter/commit/c1660100955520347ff05644d36ee4bf3532cf93))

## 0.2.1

- Handle out-of-bounds spans ([#27](https://github.com/smoelius/rewriter/pull/27))

## 0.2.0

- More revealing backup filenames ([e46df66](https://github.com/smoelius/rewriter/commit/e46df662673861c909161aa1524fcb3bbb2f1a0c))

## 0.1.1

- Update documentation ([#10](https://github.com/smoelius/rewriter/pull/10))

## 0.1.0

- Initial release
