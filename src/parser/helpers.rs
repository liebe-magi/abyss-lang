use std::sync::Arc;

use chumsky::{error::Rich, extra, prelude::*, span::SimpleSpan as ChumskySpan, text};

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

type LexerExtra<'src> = extra::Err<Rich<'src, char, ChumskySpan<usize>>>;

/// Produces a parser that skips AbySS whitespace.
pub fn abyss_whitespace<'src>() -> impl Parser<'src, &'src str, (), LexerExtra<'src>> + Clone {
    text::whitespace::<_, LexerExtra<'src>>().to(())
}

/// Helper to attach line info to AST nodes.
pub fn attach_line_info(map: &LineMap, span: SimpleSpan<usize>) -> Option<LineInfo> {
    map.line_info(span)
}

/// Replace comments with whitespace of equal length so token spans align with the original source.
pub fn scrub_comments_preserve_layout(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '/'
            && let Some(&next) = chars.peek()
        {
            if next == '/' {
                // Single-line comment: consume until newline, keep newline intact.
                result.push(' '); // replace first '/'
                chars.next(); // consume second '/'
                result.push(' ');

                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\n' {
                        result.push('\n');
                        break;
                    }
                    result.push(' ');
                }

                continue;
            } else if next == '*' {
                // Block comment: consume until closing */ while preserving newlines.
                result.push(' '); // first '/'
                chars.next(); // consume '*'
                result.push(' ');

                let mut prev = '\0';
                for c in chars.by_ref() {
                    if c == '\n' {
                        result.push('\n');
                    } else {
                        result.push(' ');
                    }

                    if prev == '*' && c == '/' {
                        break;
                    }

                    prev = c;
                }

                continue;
            }
        }

        result.push(ch);
    }

    result
}
