use crate::generator::Emitter;
use crate::property::{ContentLine, ContentLineParams};
use crate::{PARAM_DELIMITER, PARAM_VALUE_DELIMITER, VALUE_DELIMITER};
use itertools::Itertools;

pub(crate) fn split_line(line: String) -> String {
    let break_estimate = line.len().div_ceil(74);
    let mut output = String::with_capacity(line.len() + 3 * break_estimate + 2);

    let mut chars = line
        .char_indices()
        .map(|(offset, _)| offset)
        .skip(1)
        .peekable();
    let mut first_char_idx = 0;
    // Iterate over lines
    loop {
        // Find start of next line and find out if it was the last one
        let (line_boundary, last_line) = {
            let mut line_len = 0;
            loop {
                let Some(_) = chars.next() else {
                    // We are at the end, the boundary is given bv the line length (since we don't
                    // know how wide the last character is)
                    break (line.len(), true);
                };
                line_len += 1;

                if line_len == 74 {
                    // We've reached our desired length.
                    // We peek for the line boundary
                    let boundary = if let Some(&c) = chars.peek() {
                        c
                    } else {
                        line.len()
                    };
                    // char_idx currently is the start of the last character
                    break (boundary, false);
                }
            }
        };

        if first_char_idx == line_boundary {
            // There were no new characters
            break;
        }

        // This will not panic
        let left = line.split_at(line_boundary).0;
        #[cfg(test)]
        assert!(first_char_idx < line_boundary);
        output.push_str(left.split_at(first_char_idx).1);
        if last_line {
            break;
        } else {
            output.push_str("\r\n ");
        }
        first_char_idx = line_boundary;
    }

    output.push_str("\r\n");
    output
}

//
// @see: https://tools.ietf.org/html/rfc5545#section-3.3.11
//
// `text = *(TSAFE-CHAR / ":" / DQUOTE / ESCAPED-CHAR)`
//     Folded according to description above
//
// `ESCAPED-CHAR = ("\\" / "\;" / "\," / "\N" / "\n")`
//     \\ encodes \, \N or \n encodes newline
//     \; encodes ;, \, encodes ,
//
// `TSAFE-CHAR = WSP / %x21 / %x23-2B / %x2D-39 / %x3C-5B /
//              %x5D-7E / NON-US-ASCII`
//     Any character except CONTROLs not needed by the current
//     character set, DQUOTE, ";", ":", "\", ","
//
pub(crate) fn protect_param(param: &str) -> String {
    // let len = param.len() - 1;
    // starts and ends the param with quotes?
    let in_quotes = param.len() > 1 && param.starts_with('"') && param.ends_with('"');

    let mut escaped = String::with_capacity(param.len());
    let mut previous_char = None;
    for (pos, char) in param.chars().enumerate() {
        match char {
            '\n' => {
                escaped.push_str("\\n");
            }
            '"' if !in_quotes || (pos > 0 && pos < param.len() - 1) => {
                escaped.push_str("\\\"");
            }
            ';' | ':' | ',' | '\\' if !in_quotes && previous_char != Some('\\') => {
                escaped.push('\\');
                escaped.push(char)
            }
            _ => {
                escaped.push(char);
            }
        }
        previous_char = Some(char);
    }
    escaped
}

#[allow(unused)]
mod should {
    use super::{protect_param, split_line};

    #[test]
    fn split_line_multibyte() {
        let text = "a";
        assert_eq!(text, split_line(text.to_owned()).replace("\r\n", ""));
        let text = "sönderzaichän :))❗okay woow❗";
        assert_eq!(text, split_line(text.to_owned()).replace("\r\n", ""));
    }

    #[test]
    fn split_long_line() {
        let text = "The ability to return a type that is only specified by the trait it impleme\r\n \
                     n\r\n";
        assert_eq!(
            text,
            split_line(text.replace("\r\n ", "").replace("\r\n", ""))
        );
        let text = "The ability to return a type that is only specified by the trait it impleme\r\n \
                     nts is especially useful in the context closures and iterators, which we c\r\n \
                     over in Chapter 13. Closures and iterators create types that only the comp\r\n \
                     iler knows or types that are very long to specify.\r\n";
        assert_eq!(
            text,
            split_line(text.replace("\r\n ", "").replace("\r\n", ""))
        );
    }

    #[test]
    fn split_long_line_multibyte() {
        // the following text includes multibyte characters (UTF-8) at strategic places to ensure
        // split_line would panic if not multibyte aware
        let text = "DESCRIPTION:ABCDEFGHIJ\\n\\nKLMNOPQRSTUVWXYZ123456789üABCDEFGHIJKLMNOPQRS\\n\\n\r\n \
                     TUVWXYZ123456ä7890ABCDEFGHIJKLM\\n\\nNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOP\r\n \
                     QRSTUVWXöYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWX\\n\\nYZ1234567890abcdefghiÜjkl\r\n \
                     m\\nnopqrstuvwx\r\n";
        assert_eq!(
            text,
            split_line(text.replace("\r\n ", "").replace("\r\n", ""))
        );
    }

    #[test]
    fn protect_chars_in_param() {
        assert_eq!(
            protect_param("\"value: in quotes;\""),
            "\"value: in quotes;\""
        );
        assert_eq!(
            protect_param("\"value, in quotes\""),
            "\"value, in quotes\""
        );
        assert_eq!(
            protect_param("value, \"with\" something"),
            "value\\, \\\"with\\\" something"
        );
        assert_eq!(
            protect_param("\"Directory; C:\\\\Programme\""),
            "\"Directory; C:\\\\Programme\""
        );
        assert_eq!(protect_param("First\nSecond"), "First\\nSecond");
        assert_eq!(
            protect_param(
                "\"42 Plantation St.\\nBaytown\\, LA 30314\\nUnited States o\r\nf America\""
            ),
            "\"42 Plantation St.\\nBaytown\\, LA 30314\\nUnited States o\r\\nf America\""
        );
        assert_eq!(protect_param("ÄÖÜßø"), "ÄÖÜßø");
        assert_eq!(protect_param("\""), "\\\"");
        assert_eq!(protect_param("ÄÖsÜa,ßø"), "ÄÖsÜa\\,ßø");
    }
}

fn get_params(params: &ContentLineParams) -> String {
    params
        .0
        .iter()
        .map(|(name, values)| {
            let value: String = values
                .iter()
                .map(|value| protect_param(value))
                .join(&PARAM_VALUE_DELIMITER.to_string());
            format!("{name}={value}")
        })
        .join(&PARAM_DELIMITER.to_string())
}

impl Emitter for ContentLine {
    fn generate(&self) -> String {
        let mut output = self.name.to_owned();
        if !self.params.is_empty() {
            output.push(PARAM_DELIMITER);
            output.push_str(&get_params(&self.params));
        }
        output.push(VALUE_DELIMITER);
        if let Some(value) = self.value.as_ref() {
            output.push_str(value);
        }
        split_line(output)
    }
}
