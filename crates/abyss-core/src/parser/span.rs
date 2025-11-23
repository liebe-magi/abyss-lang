use std::ops::Range;

use chumsky::span::Span as ChumskySpan;

/// A minimal span implementation compatible with chumsky's [`Span`] trait.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimpleSpan<T> {
    start: T,
    end: T,
}

impl<T: Copy + Ord> SimpleSpan<T> {
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> T {
        self.start
    }

    pub fn end(&self) -> T {
        self.end
    }

    pub fn into_range(self) -> Range<T> {
        self.start..self.end
    }
}

impl<T: Copy + Ord> From<Range<T>> for SimpleSpan<T> {
    fn from(range: Range<T>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl<T: Copy + Ord> From<SimpleSpan<T>> for Range<T> {
    fn from(span: SimpleSpan<T>) -> Self {
        span.into_range()
    }
}

impl<T: Copy + Ord> ChumskySpan for SimpleSpan<T> {
    type Context = ();
    type Offset = T;

    fn new(_: Self::Context, range: Range<Self::Offset>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}
