#![cfg_attr(docsrs, feature(doc_cfg))]

use std::ops::Range;

use bytes::Bytes;

pub mod headers;

use crate::headers::{
    content_range::{Bound, HttpContentRange, Unsatisfiable},
    range::HttpRange,
};

/// Returns a [`BodyRange`] of [`Bytes`] if the provided [`HttpRange`] is satisfiable, otherwise it returns [`UnsatisfiableRange`].
///
/// [`HttpRange`]: crate::headers::range::HttpRange
pub fn serve_file_with_http_range(
    body: Bytes,
    http_range: Option<HttpRange>,
) -> Result<BodyRange<Bytes>, UnsatisfiableRange> {
    let size = u64::try_from(body.len()).expect("we do not support 128bit usize");

    let content_range = file_range(size, http_range)?;

    let start = usize::try_from(content_range.range.start).expect("u64 doesn't fit usize");
    let end = usize::try_from(content_range.range.end).expect("u64 doesn't fit usize");

    Ok(BodyRange {
        body: body.slice(start..end),
        header: content_range.header,
    })
}

/// Returns a [`ContentRange`] if the provided [`HttpRange`] is satisfiable, otherwise it returns [`UnsatisfiableRange`].
///
/// [`HttpRange`]: crate::headers::range::HttpRange
pub fn file_range(
    size: u64,
    http_range: Option<HttpRange>,
) -> Result<ContentRange, UnsatisfiableRange> {
    let Some(http_range) = http_range else {
        return Ok(ContentRange {
            header: None,
            range: 0..size,
        });
    };

    let range = match http_range {
        HttpRange::StartingPoint(start) if start < size => start..size,
        HttpRange::Range(range) if range.start() < size => {
            range.start()..range.end().saturating_add(1).min(size)
        }
        HttpRange::Suffix(suffix) if suffix > 0 && size > 0 => size.saturating_sub(suffix)..size,
        _ => {
            let content_range = HttpContentRange::Unsatisfiable(Unsatisfiable::new(size));
            return Err(UnsatisfiableRange(content_range));
        }
    };

    let content_range =
        HttpContentRange::Bound(Bound::new(range.start..=range.end - 1, Some(size)).unwrap());

    Ok(ContentRange {
        header: Some(content_range),
        range,
    })
}

/// A container for the payload slice and the optional `Content-Range` header.
///
/// The header is `None` only if the body was not sliced.
///
/// If the `axum` feature is enabled this struct also implements `IntoResponse`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyRange<T> {
    body: T,
    header: Option<HttpContentRange>,
}

impl<T> BodyRange<T> {
    /// Returns the sliced body.
    pub fn body(&self) -> &T {
        &self.body
    }

    pub fn into_body(self) -> T {
        self.body
    }

    /// Returns an option of [`HttpContentRange`].
    /// If it's None the provided [`HttpRange`] was None too.
    pub fn header(&self) -> Option<HttpContentRange> {
        self.header
    }
}

/// A container for the payload range and the optional `Content-Range` header.
///
/// The header is `None` only if the body was not sliced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentRange {
    header: Option<HttpContentRange>,
    range: Range<u64>,
}

impl ContentRange {
    /// Returns an option of [`HttpContentRange`].
    /// If it's None the provided [`HttpRange`] was None too.
    pub fn header(&self) -> Option<HttpContentRange> {
        self.header
    }

    /// Returns a [`Range`] of `u64` useful to manually slice the response body.
    pub fn range(&self) -> &Range<u64> {
        &self.range
    }
}

/// An unsatisfiable range request.
///
/// If the `axum` feature is enabled this struct also implements `IntoResponse`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsatisfiableRange(HttpContentRange);

impl UnsatisfiableRange {
    /// Returns the [`HttpContentRange`] header.
    pub fn header(&self) -> HttpContentRange {
        self.0
    }
}

#[cfg(feature = "axum")]
mod axum {
    use crate::{BodyRange, UnsatisfiableRange};

    use axum_core::response::{IntoResponse, Response};
    use bytes::Bytes;
    use http::{HeaderValue, StatusCode, header::CONTENT_RANGE};

    impl IntoResponse for BodyRange<Bytes> {
        fn into_response(self) -> Response {
            match self.header {
                Some(range) => (
                    StatusCode::PARTIAL_CONTENT,
                    [(CONTENT_RANGE, HeaderValue::from(&range))],
                    self.body,
                )
                    .into_response(),
                None => (StatusCode::OK, self.body).into_response(),
            }
        }
    }

    impl IntoResponse for UnsatisfiableRange {
        fn into_response(self) -> Response {
            (
                StatusCode::RANGE_NOT_SATISFIABLE,
                [(CONTENT_RANGE, HeaderValue::from(&self.0))],
            )
                .into_response()
        }
    }
}
