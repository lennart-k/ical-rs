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
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! let buf = BufReader::new(File::open("./tests/resources/vcard_input.vcf").unwrap());
//!
//! let reader = ical::LineReader::new(buf);
//!
//! for line in reader {
//!     println!("{}", line);
//! }
//! ```

use std::fmt;
use std::io::{BufRead, Lines};
use std::iter::{Iterator, Peekable};

/// An unfolded raw line.
///
/// Its inner is only a raw line from the file. No parsing or checking have
/// been made yet.
#[derive(Debug, Clone, Default)]
pub struct Line {
    inner: String,
    number: usize,
}

impl Line {
    pub fn new(line: String, line_number: usize) -> Line {
        Line {
            inner: line,
            number: line_number,
        }
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    pub fn number(&self) -> usize {
        self.number
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {}: {}", self.number, self.inner)
    }
}

/// Take a `BufRead` and return the unfolded `Line`.
pub struct LineReader<B: BufRead> {
    lines: Peekable<Lines<B>>,
    number: usize,
}

impl<B: BufRead> LineReader<B> {
    /// Return a new `LineReader` from a `Reader`.
    pub fn new(reader: B) -> LineReader<B> {
        LineReader {
            lines: reader.lines().peekable(),
            number: 0,
        }
    }
}

impl<B: BufRead> Iterator for LineReader<B> {
    type Item = Line;

    fn next(&mut self) -> Option<Line> {
        let (mut new_line, line_number) = loop {
            let line = self.lines.next()?.ok()?;
            self.number += 1;
            if !line.is_empty() {
                break (line.trim_end().to_string(), self.number);
            }
        };

        loop {
            let Some(Ok(next)) = self.lines.next_if(|line| {
                line.as_ref()
                    .ok()
                    .map(|line| line.starts_with(' ') || line.starts_with('\t') || line.is_empty())
                    .unwrap_or_default()
            }) else {
                break;
            };
            self.number += 1;
            if !next.is_empty() {
                // String cannot be empty so this cannot panic
                new_line.push_str(next.split_at(1).1);
            }
        }

        if new_line.is_empty() {
            None
        } else {
            Some(Line::new(new_line, line_number))
        }
    }
}
