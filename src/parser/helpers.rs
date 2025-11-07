use std::sync::Arc;

use chumsky::prelude::*;
use chumsky::text;

use crate::ast::LineInfo;

use super::SimpleSpan;

/// Represents a mapping between byte offsets and (line, column) positions.
#[derive(Debug, Clone)]
pub struct LineMap {
    line_starts: Arc<Vec<usize>>,
}

impl LineMap {
    pub fn new(source: &str) -> Self {
        let mut starts = Vec::with_capacity(source.lines().count() + 1);
        starts.push(0);
        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                starts.push(idx + ch.len_utf8());
            }
        }
        LineMap {
            line_starts: Arc::new(starts),
        }
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.line_starts[line_idx];
        (line_idx + 1, offset - line_start + 1)
    }

    pub fn line_info(&self, span: SimpleSpan<usize>) -> Option<LineInfo> {
        let (line, column) = self.line_col(span.start());
        Some(LineInfo::new(line, column))
    }
}

/// Produces a parser that skips AbySS whitespace and comments.
pub fn abyss_whitespace() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    let spaces = text::whitespace().at_least(1).ignored();

    let line_comment = just("//")
        .ignore_then(filter(|c| *c != '\n').repeated())
        .then_ignore(just('\n').or_not())
        .ignored();

    let block_comment = just("/*")
        .ignore_then(just("*/").not().ignore_then(any()).repeated())
        .then_ignore(just("*/"))
        .ignored();

    spaces
        .or(line_comment)
        .or(block_comment)
        .repeated()
        .ignored()
}

/// Helper to attach line info to AST nodes.
pub fn attach_line_info(map: &LineMap, span: SimpleSpan<usize>) -> Option<LineInfo> {
    map.line_info(span)
}
