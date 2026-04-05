#[cfg(feature = "axum")]
use std::convert::Infallible;
use std::str::FromStr;

use http::HeaderValue;

use crate::headers::range::HttpRange;

/// A typed HTTP `If-Range` header.
///
/// Per [RFC 9110 Section 13.1.5], `If-Range` can contain either an HTTP-date
/// or an entity-tag. When present alongside a `Range` header, the server must
/// evaluate the validator against the current representation:
///
/// - If the validator **matches**, the `Range` is honored (206 Partial Content).
/// - If the validator **does not match**, the `Range` is ignored and the full
///   representation is served (200 OK).
///
/// [RFC 9110 Section 13.1.5]: https://www.rfc-editor.org/rfc/rfc9110#section-13.1.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IfRange {
    /// An HTTP-date validator (the raw header value, to be compared with `Last-Modified`).
    Date(HeaderValue),
    /// An entity-tag validator (the raw header value, to be compared with `ETag`).
    ETag(HeaderValue),
}

impl IfRange {
    /// Evaluates the `If-Range` condition and returns the [`HttpRange`] only if
    /// the condition holds.
    ///
    /// - `range`: the parsed `Range` header value.
    /// - `last_modified`: the current `Last-Modified` header of the representation.
    /// - `etag`: the current `ETag` header of the representation.
    ///
    /// Returns `Some(range)` if the validator matches (the range should be
    /// honored), or `None` if it does not (the full representation should be
    /// served).
    ///
    /// Per [RFC 9110 Section 13.1.5], the comparison uses the raw header values:
    /// - For dates, the `If-Range` value must be an **exact byte-for-byte match**
    ///   of the `Last-Modified` header value.
    /// - For entity-tags, the `If-Range` value must be a **strong comparison**
    ///   match against the `ETag` header value. Weak entity-tags never match.
    ///
    /// [RFC 9110 Section 13.1.5]: https://www.rfc-editor.org/rfc/rfc9110#section-13.1.5
    pub fn evaluate(
        &self,
        range: HttpRange,
        last_modified: Option<&HeaderValue>,
        etag: Option<&HeaderValue>,
    ) -> Option<HttpRange> {
        let matches = match self {
            IfRange::Date(date) => last_modified.is_some_and(|lm| lm == date),
            IfRange::ETag(tag) => etag.is_some_and(|et| strong_etag_eq(tag, et)),
        };

        if matches { Some(range) } else { None }
    }
}

/// Performs a strong comparison of two entity-tags.
///
/// Per [RFC 9110 Section 8.8.3.2], two entity-tags are strongly equivalent if
/// both are **not** weak and their opaque-tags match character by character.
///
/// [RFC 9110 Section 8.8.3.2]: https://www.rfc-editor.org/rfc/rfc9110#section-8.8.3.2
fn strong_etag_eq(a: &HeaderValue, b: &HeaderValue) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();

    // Weak tags (W/"...") never match in a strong comparison
    if a.starts_with(b"W/") || b.starts_with(b"W/") {
        return false;
    }

    a == b
}

impl FromStr for IfRange {
    type Err = InvalidIfRange;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(InvalidIfRange);
        }

        let value = HeaderValue::from_str(s).map_err(|_| InvalidIfRange)?;

        // Per RFC 9110 Section 13.1.5, the field value is either an entity-tag
        // or an HTTP-date. Entity-tags start with `"` or `W/"`.
        if s.starts_with('"') || s.starts_with("W/\"") {
            Ok(IfRange::ETag(value))
        } else {
            Ok(IfRange::Date(value))
        }
    }
}

impl TryFrom<&HeaderValue> for IfRange {
    type Error = InvalidIfRange;

    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        value.to_str().map_err(|_| InvalidIfRange)?.parse::<Self>()
    }
}

/// An error returned when parsing an `If-Range` header fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid If-Range header")]
pub struct InvalidIfRange;

#[cfg(feature = "axum")]
impl<S> axum_core::extract::OptionalFromRequestParts<S> for IfRange
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let if_range = parts
            .headers
            .get(http::header::IF_RANGE)
            .and_then(|v| IfRange::try_from(v).ok());
        Ok(if_range)
    }
}
