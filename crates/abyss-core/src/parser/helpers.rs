use chumsky::{error::Rich, extra, prelude::*, span::SimpleSpan as ChumskySpan, text};

type LexerExtra<'src> = extra::Err<Rich<'src, char, ChumskySpan<usize>>>;

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
