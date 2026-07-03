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

/// Scan `source` for the byte regions occupied by comments, in order.
///
/// The scanner is rune-literal aware: `//` and `/*` inside a string are
/// text, not comment openers, and the lexer's escape rules are honoured
/// so `"\""` does not end the literal early. A line comment spans from
/// `//` to just before the newline (or EOF); a block comment spans from
/// `/*` through the closing `*/` (or EOF when unterminated). Region
/// boundaries always fall on `char` boundaries.
///
/// This single scanner backs both [`scrub_comments_preserve_layout`] and
/// [`collect_comments`], so their views of the source can never diverge.
fn comment_regions(source: &str) -> Vec<(usize, usize)> {
    let mut regions = Vec::new();
    let mut chars = source.char_indices().peekable();
    let mut in_string = false;

    while let Some((idx, ch)) = chars.next() {
        if in_string {
            match ch {
                // Skip the escaped character so `\"` stays inside the
                // literal — mirroring the lexer's escape handling.
                '\\' => {
                    chars.next();
                }
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '/' => {
                let Some(&(_, next)) = chars.peek() else {
                    continue;
                };
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
                    regions.push((idx, end));
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
                    regions.push((idx, end));
                }
            }
            _ => {}
        }
    }

    regions
}

/// Collect every comment in `source` with its byte span, in source order.
pub fn collect_comments(source: &str) -> Vec<SourceComment> {
    comment_regions(source)
        .into_iter()
        .map(|(start, end)| SourceComment {
            span: Span::new(start, end),
            text: source[start..end].trim_end().to_string(),
        })
        .collect()
}

/// Replace comments with whitespace of equal **byte** length so token spans
/// align exactly with the original source. Blanking is byte-wise (newlines
/// inside block comments are kept) — a multi-byte character in a comment
/// becomes that many spaces, so subsequent offsets never shift.
pub fn scrub_comments_preserve_layout(source: &str) -> String {
    let mut bytes = source.as_bytes().to_vec();
    for (start, end) in comment_regions(source) {
        for byte in &mut bytes[start..end] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    // Regions start/end on char boundaries and every blanked byte becomes
    // ASCII space, so the result is valid UTF-8 by construction.
    String::from_utf8(bytes).expect("blanking comments preserves UTF-8 validity")
}

/// Produces a parser that skips AbySS whitespace.
pub fn abyss_whitespace<'src>() -> impl Parser<'src, &'src str, (), LexerExtra<'src>> + Clone {
    text::whitespace::<_, LexerExtra<'src>>().to(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slashes_inside_rune_literals_are_not_comments() {
        let source = r#"forge url: rune = "http://example.com"; // real comment"#;
        let scrubbed = scrub_comments_preserve_layout(source);
        assert!(scrubbed.contains(r#""http://example.com""#));
        assert!(!scrubbed.contains("real comment"));

        let comments = collect_comments(source);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "// real comment");
    }

    #[test]
    fn block_comment_opener_inside_string_is_text() {
        let source = r#"unveil("/* not a comment */");"#;
        assert_eq!(scrub_comments_preserve_layout(source), source);
        assert!(collect_comments(source).is_empty());
    }

    #[test]
    fn escaped_quote_does_not_end_the_literal() {
        let source = r#"forge s: rune = "say \"hi\" // still text";"#;
        assert_eq!(scrub_comments_preserve_layout(source), source);
        assert!(collect_comments(source).is_empty());
    }

    #[test]
    fn quote_inside_comment_does_not_open_a_string() {
        let source = "// a \"quoted\" note\nforge x: arcana = 1; // b\n";
        let comments = collect_comments(source);
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].text, "// a \"quoted\" note");
        assert_eq!(comments[1].text, "// b");
        let scrubbed = scrub_comments_preserve_layout(source);
        assert!(scrubbed.contains("forge x: arcana = 1;"));
        assert!(!scrubbed.contains("note"));
    }

    #[test]
    fn multibyte_comment_blanking_preserves_byte_offsets() {
        let source = "// コメント\nforge x: arcana = 1;\n";
        let scrubbed = scrub_comments_preserve_layout(source);
        assert_eq!(scrubbed.len(), source.len(), "byte length must not shift");
        let forge_at = source.find("forge").unwrap();
        assert_eq!(scrubbed.find("forge").unwrap(), forge_at);
    }

    #[test]
    fn unterminated_block_comment_runs_to_eof() {
        let source = "forge x: arcana = 1; /* dangling";
        let comments = collect_comments(source);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "/* dangling");
        assert!(scrub_comments_preserve_layout(source).starts_with("forge x: arcana = 1;"));
    }
}
