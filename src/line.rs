//! Read and unfold a line from a `BufRead`.
//!
//! Individual lines within vCard are delimited by the [RFC5322] line
//! break, which is a CRLF sequence (U+000D followed by U+000A).  Long
//! logical lines of text can be split into a multiple-physical-line
//! representation using the following folding technique.  Content lines
//! SHOULD be folded to a maximum width of 75 octets, excluding the line
//! break.  Multi-octet characters MUST remain contiguous.  The rationale
//! for this folding process can be found in [RFC5322], Section 2.1.1.
//!
//! A logical line MAY be continued on the next physical line anywhere
//! between two characters by inserting a CRLF immediately followed by a
//! single white space character (space (U+0020) or horizontal tab
//! (U+0009)).  The folded line MUST contain at least one character.  Any
//! sequence of CRLF followed immediately by a single white space
//! character is ignored (removed) when processing the content type.
//!
//! [RFC5322]: https://tools.ietf.org/html/rfc5322
//! # Examples
//!
//! ```toml
//! [dependencies.ical]
//! version = "0.3.*"
//! default-features = false
//! features = ["line-reader"]
//! ```
//!
//! ```rust
//! extern crate ical;
//!
//! use std::fs::read_to_string;
//!
//! let buf = read_to_string("./tests/resources/vcard_input.vcf")
//!     .unwrap();
//!
//! let reader = ical::LineReader::from_slice(buf.as_bytes());
//!
//! for line in reader {
//!     println!("{:?}", line);
//! }
//! ```

use std::borrow::Cow;
use std::fmt;
use std::iter::{Iterator, Peekable};
use std::str::Utf8Error;
use std::string::FromUtf8Error;

/// An unfolded raw line.
///
/// Its inner is only a raw line from the file. No parsing or checking have
/// been made yet.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Line<'a> {
    inner: Cow<'a, str>,
    number: usize,
}

impl<'a> Line<'a> {
    pub fn new(line: Cow<'a, str>, line_number: usize) -> Line<'a> {
        Line {
            inner: line,
            number: line_number,
        }
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_ref()
    }

    pub fn number(&self) -> usize {
        self.number
    }
}

impl<'a> fmt::Display for Line<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {}: {}", self.number, self.inner)
    }
}

// An iterator over lines that works with binary content
// std::io::Lines is not applicable since multi-octet sequences might be wrapped over multiple lines
#[derive(Debug)]
pub struct BytesLines<'a>(&'a [u8]);

impl<'a> Iterator for BytesLines<'a> {
    type Item = Cow<'a, [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.iter().position(|val| val == &b'\n') {
            Some(pos) => {
                // Is there a multi-octet character that ends with \r=0x0d?
                let line_end = if pos > 0 && self.0[pos - 1] == b'\r' {
                    pos - 1
                } else {
                    pos
                };
                let line = &self.0[..line_end];

                // That's the position after the line break \n
                self.0 = self.0.split_at(pos + 1).1;
                Some(Cow::Borrowed(line))
            }
            None if !self.0.is_empty() => {
                let line = self.0;
                self.0 = &[];
                Some(Cow::Borrowed(line))
            }
            None => None,
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum LineError {
    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
}

/// Take a `BufRead` and return the unfolded `Line`.
pub struct LineReader<'a, I: Iterator<Item = Cow<'a, [u8]>>> {
    lines: Peekable<I>,
    number: usize,
}

impl<'a> LineReader<'a, BytesLines<'a>> {
    /// Return a new `LineReader` from a `Reader`.
    pub fn from_slice(reader: &'a [u8]) -> LineReader<'a, BytesLines<'a>> {
        LineReader {
            lines: BytesLines(reader).peekable(),
            number: 0,
        }
    }
}

impl<'a, T: Iterator<Item = Cow<'a, [u8]>>> Iterator for LineReader<'a, T> {
    type Item = Result<Line<'a>, LineError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (mut new_line, line_number) = loop {
            let line = self.lines.next()?;
            self.number += 1;
            if !line.is_empty() {
                break (line, self.number);
            }
        };

        loop {
            let Some(next) = self.lines.next_if(|line| {
                line.starts_with(b" ") || line.starts_with(b"\t") || line.is_empty()
            }) else {
                break;
            };
            self.number += 1;
            if !next.is_empty() {
                // String cannot be empty so this cannot panic
                new_line.to_mut().extend_from_slice(next.split_at(1).1);
            }
        }

        let new_line = match new_line {
            Cow::Owned(bytes) => Cow::Owned(match String::from_utf8(bytes) {
                Ok(val) => val,
                Err(err) => return Some(Err(err.into())),
            }),
            Cow::Borrowed(slice) => Cow::Borrowed(match str::from_utf8(slice) {
                Ok(val) => val,
                Err(err) => return Some(Err(err.into())),
            }),
        };

        dbg!(&new_line);
        if new_line.is_empty() {
            None
        } else {
            Some(Ok(Line::new(new_line, line_number)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Line, LineReader};
    use rstest::rstest;

    #[rstest]
    #[case("", vec![])]
    #[case("\n", vec![])]
    #[case("asd", vec![Line{inner: "asd".into(), number: 1}])]
    #[case("asd\r\n  ok", vec![Line{inner: "asd ok".into(), number: 1}])]
    #[case("asd with linebreak\r\n \r\n  ok", vec![Line{inner: "asd with linebreak ok".into(), number: 1}])]
    #[case("weird with linebreak\r\n\r\n  ok", vec![Line{inner: "weird with linebreak ok".into(), number: 1}])]
    #[case("line1\r\n\r\nline2", vec![Line{inner: "line1".into(), number: 1}, Line{inner: "line2".into(), number: 3}])]
    fn test_line_reader(#[case] input: &str, #[case] lines: Vec<Line>) {
        let parsed_lines = LineReader::from_slice(input.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(parsed_lines, lines);
    }
}
