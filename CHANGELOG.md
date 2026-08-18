# Changelog

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
