mod impls;

use impls::LazyRewriter;

#[cfg(feature = "check-rewrites")]
use impls::EagerRewriter;

pub trait Interface {
    fn contents(self) -> String;
    fn rewrite(&mut self, start: usize, end: usize, replacement: &str) -> String;
}

#[derive(Debug)]
pub struct OffsetBasedRewriter<'original> {
    lazy: LazyRewriter<'original>,

    #[cfg(feature = "check-rewrites")]
    eager: EagerRewriter,
}

impl<'original> OffsetBasedRewriter<'original> {
    pub fn new(original: &'original str) -> Self {
        Self {
            lazy: LazyRewriter::new(original),

            #[cfg(feature = "check-rewrites")]
            eager: EagerRewriter::new(original),
        }
    }
}

impl Interface for OffsetBasedRewriter<'_> {
    #[allow(clippy::let_and_return)]
    fn contents(self) -> String {
        let contents = self.lazy.contents();

        #[cfg(feature = "check-rewrites")]
        {
            let contents_comparator = self.eager.contents();
            assert_eq!(contents, contents_comparator);
        }

        contents
    }

    fn rewrite(&mut self, start: usize, end: usize, replacement: &str) -> String {
        let replaced = self.lazy.rewrite(start, end, replacement);

        #[cfg(feature = "check-rewrites")]
        assert_eq!(replaced, self.eager.rewrite(start, end, replacement));

        replaced
    }
}
