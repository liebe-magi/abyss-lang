use chumsky::{error::Rich, extra, prelude::*, span::SimpleSpan as ChumskySpan, text};

use crate::span::Span;

type LexerExtra<'src> = extra::Err<Rich<'src, char, ChumskySpan<usize>>>;

/// A comment lifted out of the original source, with the byte span it
/// occupied. Produced by [`collect_comments`]; consumed by the formatter's
/// comment-preserving `format_program` to re-emit comments alongside the
/// statements they belonged to.
#[derive(Debug, Clone)]
pub struct SourceComment {
    pub span: Span,
    pub text: String,
}

/// Collect every comment in `source` with its byte span, in source order.
///
/// Uses the same scan as [`scrub_comments_preserve_layout`], so the spans
/// line up exactly with the regions the scrubber blanks out before lexing.
pub fn collect_comments(source: &str) -> Vec<SourceComment> {
    let mut comments = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if ch == '/'
            && let Some(&(_, next)) = chars.peek()
        {
            if next == '/' {
                chars.next(); // consume second '/'
                let mut end = source.len();
                while let Some(&(c_idx, c)) = chars.peek() {
                    if c == '\n' {
                        end = c_idx;
                        break;
                    }
                    chars.next();
                }
                comments.push(SourceComment {
                    span: Span::new(idx, end),
                    text: source[idx..end].trim_end().to_string(),
                });
                continue;
            } else if next == '*' {
                chars.next(); // consume '*'
                let mut end = source.len();
                let mut prev = '\0';
                for (c_idx, c) in chars.by_ref() {
                    if prev == '*' && c == '/' {
                        end = c_idx + c.len_utf8();
                        break;
                    }
                    prev = c;
                }
                comments.push(SourceComment {
                    span: Span::new(idx, end),
                    text: source[idx..end].to_string(),
                });
                continue;
            }
        }
    }

    comments
}

/// Produces a parser that skips AbySS whitespace.
pub fn abyss_whitespace<'src>() -> impl Parser<'src, &'src str, (), LexerExtra<'src>> + Clone {
    text::whitespace::<_, LexerExtra<'src>>().to(())
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
