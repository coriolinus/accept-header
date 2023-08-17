use std::{fmt, str::FromStr};

use bstr::BString;
use headers_core::{self, Header, HeaderName, HeaderValue};
use mime::Mime;
use noisy_float::types::R32;

mod media_type;
use media_type::MediaType;

/// `Quality` is a real number in the (inclusive) range `[0.0..=1.0]`.
pub type Quality = R32;

/// This header lets the client specify what sort of content it wants to receive.
///
/// See [its specification in RFC9110](https://www.rfc-editor.org/rfc/rfc9110#name-accept).
#[derive(Debug, Clone)]
pub struct Accept {
    /// This is always stored in descending order of quality.
    media_types: Vec<MediaType>,
}

impl Accept {
    pub const HEADER_NAME: &str = "accept";

    /// Parse the complete header, including the header name.
    ///
    /// Parsing will fail unless the header starts with (case insensitive) `accept:`.
    pub fn parse(header: &str) -> Result<Self, ParseError> {
        const HEADER_IDX: usize = Accept::HEADER_NAME.len();
        const HEADER_COLON_IDX: usize = HEADER_IDX + 1;

        // work on the bytes level to avoid panics for multi-byte characters
        let expect_substr = &header.as_bytes()[..HEADER_IDX.min(header.len())];
        if !expect_substr.eq_ignore_ascii_case(Self::HEADER_NAME.as_bytes()) {
            return Err(ParseError::WrongHeader(expect_substr.into()));
        }

        if !header.is_char_boundary(HEADER_COLON_IDX) {
            return Err(ParseError::BodyIndexNotOnCharacterBoundary);
        }

        Self::parse_body(&header[HEADER_COLON_IDX..])
    }

    /// Parse the header body, excluding the header name.
    pub fn parse_body(body: &str) -> Result<Self, ParseError> {
        let mut media_types = Vec::new();
        for input in body.split(',') {
            let input = input.trim();
            let mime = input
                .parse::<Mime>()
                .map_err(|err| ParseError::FailedToParseMediaType {
                    input: input.into(),
                    err,
                })?;
            media_types.push(mime.into())
        }

        // maintain descending order by relevance
        media_types.sort();
        media_types.reverse();

        Ok(Self { media_types })
    }

    /// Iterate over the acceptable media types, from highest to lowest priority.
    pub fn media_types(&self) -> impl '_ + Iterator<Item = &MediaType> {
        self.media_types.iter()
    }

    /// Create a formatter which formats only the body of this type.
    fn fmt_body(&self) -> FormatBody {
        FormatBody {
            media_types: &self.media_types,
        }
    }

    /// Emit the header's body without its header.
    pub fn body_to_string(&self) -> String {
        self.fmt_body().to_string()
    }
}

impl FromStr for Accept {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for Accept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "accept: ")?;
        self.fmt_body().fmt(f)
    }
}

// best-guess implementation; see <https://github.com/hyperium/headers/issues/144>
impl Header for Accept {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static(Accept::HEADER_NAME);
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers_core::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers_core::Error::invalid)?;
        let value_str = value.to_str().map_err(|_| headers_core::Error::invalid())?;
        Accept::parse_body(value_str).map_err(|_| headers_core::Error::invalid())
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let header_value = HeaderValue::from_str(&self.body_to_string())
            .expect("header canonical form includes only visible ascii chars");
        values.extend(::std::iter::once(header_value));
    }
}

#[derive(Debug, Clone)]
struct FormatBody<'a> {
    media_types: &'a [MediaType],
}

impl<'a> fmt::Display for FormatBody<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for media_type in self.media_types {
            if first {
                first = false;
            } else {
                f.write_str(", ")?;
            }
            media_type.fmt(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("wrong header name: expect \"accept\"; have \"{0}\"")]
    WrongHeader(BString),
    #[error("body index does not fall on a character boundary")]
    BodyIndexNotOnCharacterBoundary,
    #[error("failed to parse media type \"{input}\"")]
    FailedToParseMediaType {
        input: String,
        #[source]
        err: mime::FromStrError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    fn perform_test(input: &str, expect: &[(&str, f32, Option<(&str, &str)>)]) {
        let accept = Accept::parse(input).unwrap();
        dbg!(&accept);
        for (media_type, (essence_str, quality, expect_param)) in
            accept.media_types().zip_eq(expect.iter().copied())
        {
            assert_eq!(media_type.mime.essence_str(), essence_str);
            assert_eq!(media_type.quality_factor(), Quality::new(quality));
            if let Some((key, value)) = expect_param {
                assert_eq!(media_type.mime.get_param(key).unwrap(), value);
            }
        }
    }

    #[test]
    fn rfc_example_audio() {
        let input = "Accept: audio/*; q=0.2, audio/basic";
        let expect = &[("audio/basic", 1.0, None), ("audio/*", 0.2, None)];
        perform_test(input, expect);
    }

    #[test]
    fn rfc_example_more_elaborate() {
        let input = "Accept: text/plain; q=0.5, text/html,
        text/x-dvi; q=0.8, text/x-c";
        let expect = &[
            ("text/html", 1.0, None),
            ("text/x-c", 1.0, None),
            ("text/x-dvi", 0.8, None),
            ("text/plain", 0.5, None),
        ];
        perform_test(input, expect);
    }

    #[test]
    fn rfc_example_media_ranges() {
        let input = "Accept: text/*, text/plain, text/plain;format=flowed, */*";
        let expect = &[
            ("text/plain", 1.0, Some(("format", "flowed"))),
            ("text/plain", 1.0, None),
            ("text/*", 1.0, None),
            ("*/*", 1.0, None),
        ];
        perform_test(input, expect);

        let accept = Accept::parse(input).unwrap();
        let second = accept.media_types().nth(1).unwrap();
        assert_eq!(second.mime.get_param("format"), None);
    }

    #[test]
    fn rfc_example_media_type_quality_factor() {
        let input = "Accept: text/*;q=0.3, text/plain;q=0.7, text/plain;format=flowed,
        text/plain;format=fixed;q=0.4, */*;q=0.5";
        let expect = &[
            ("text/plain", 1.0, Some(("format", "flowed"))),
            ("text/plain", 0.7, None),
            ("*/*", 0.5, None),
            ("text/plain", 0.4, Some(("format", "fixed"))),
            ("text/*", 0.3, None),
        ];
        perform_test(input, expect);
    }

    #[test]
    fn empty() {
        let input = "Accept:";
        let err = Accept::parse(input).unwrap_err();
        let ParseError::FailedToParseMediaType { err, .. } = err else {panic!("wrong err variant")};
        dbg!(&err, err.to_string());
        let err_str = err.to_string();
        assert!(err_str.contains("a slash"));
        assert!(err_str.contains("was missing"));
    }

    #[test]
    fn singleton() {
        let input = "Accept: application/json";
        let expect = &[("application/json", 1.0, None)];
        perform_test(input, expect);
    }
}
