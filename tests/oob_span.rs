use rewriter::{LineColumn, Rewriter, Span};

// smoelius:
//
// (1,0) X (1,1)
// (2,0)

#[test]
fn oob_span() {
    const PAIRS: &[(LineColumn, LineColumn)] = &[
        (
            LineColumn { line: 1, column: 1 },
            LineColumn { line: 1, column: 2 },
        ),
        (
            LineColumn { line: 2, column: 0 },
            LineColumn { line: 2, column: 1 },
        ),
        (
            LineColumn { line: 1, column: 0 },
            LineColumn { line: 1, column: 2 },
        ),
        (
            LineColumn { line: 1, column: 0 },
            LineColumn { line: 2, column: 1 },
        ),
    ];
    for &(start, end) in PAIRS {
        let span = Span::new(start, end);
        let _: String = Rewriter::new("X").rewrite(&span, "Y");
    }
}
