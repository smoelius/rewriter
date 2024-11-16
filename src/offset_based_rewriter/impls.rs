use super::Interface;

#[derive(Debug)]
pub struct LazyRewriter<'original> {
    original: &'original str,
    rewritten: String,
    offset: usize,
}

#[derive(Debug)]
pub struct EagerRewriter {
    rewritten: String,
    delta: isize,
}

impl<'original> LazyRewriter<'original> {
    pub fn new(original: &'original str) -> Self {
        Self {
            original,
            rewritten: String::new(),
            offset: 0,
        }
    }
}

impl EagerRewriter {
    #[allow(dead_code)]
    pub fn new(original: &str) -> Self {
        Self {
            rewritten: original.to_owned(),
            delta: 0,
        }
    }
}

impl Interface for LazyRewriter<'_> {
    fn contents(mut self) -> String {
        self.rewritten += &self.original[self.offset..];

        self.rewritten
    }

    fn rewrite(&mut self, start: usize, end: usize, replacement: &str) -> String {
        assert!(self.offset <= start);

        self.rewritten += &self.original[self.offset..start];
        self.rewritten += replacement;

        self.offset = end;

        String::from(&self.original[self.offset - (end - start)..self.offset])
    }
}

impl Interface for EagerRewriter {
    fn contents(self) -> String {
        self.rewritten
    }

    #[allow(clippy::cast_possible_wrap)]
    fn rewrite(&mut self, start: usize, end: usize, replacement: &str) -> String {
        let start = usize::try_from(start as isize + self.delta).unwrap();
        let end = usize::try_from(end as isize + self.delta).unwrap();

        let prefix = &self.rewritten.as_bytes()[..start];
        let replaced = &self.rewritten.as_bytes()[start..end];
        let suffix = &self.rewritten.as_bytes()[end..];

        let replaced = String::from_utf8(replaced.to_vec()).expect("`replaced` is not valid UTF-8");

        self.rewritten = String::from_utf8(prefix.to_vec()).expect("`prefix` is not valid UTF-8")
            + replacement
            + std::str::from_utf8(suffix).expect("`suffix` is not valid UTF-8");

        self.delta += replacement.len() as isize - end as isize + start as isize;

        replaced
    }
}
