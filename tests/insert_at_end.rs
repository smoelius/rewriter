use rewriter::{LineColumn, Rewriter, Span};

// An empty span at the end of the input is valid. Rewriting it should insert the replacement, not
// discard it.
#[test]
fn insert_at_end() {
    let end = LineColumn { line: 1, column: 1 };
    let span = Span::new(end, end);

    let mut rewriter = Rewriter::new("X");
    let replaced = rewriter.rewrite(&span, "Y").unwrap();

    assert_eq!("", replaced);
    assert_eq!("XY", rewriter.contents());
}
