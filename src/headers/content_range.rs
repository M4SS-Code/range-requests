use std::{
    fmt::{self, Display},
    ops::RangeInclusive,
    str::FromStr,
};

use http::HeaderValue;

use crate::headers::{
    InvalidHttpU64, InvalidOrderedRange, OrderedRange, ParseHttpRangeOrContentRangeError, UNIT,
    range::HttpRange, u64_unprefixed_parse,
};

/// A typed HTTP `Content-Range` header that only supports a __single__ range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpContentRange {
    Bound(Bound),
    Unsatisfiable(Unsatisfiable),
}

impl HttpContentRange {
    /// Checks whether this `Content-Range` matches the expected [`HttpRange`].
    ///
    /// [`HttpRange`]: crate::headers::range::HttpRange
    pub fn matches_requested_range(&self, expected_range: HttpRange) -> bool {
        match (expected_range, self) {
            (HttpRange::StartingPoint(start), HttpContentRange::Bound(Bound { range, .. })) => {
                start == range.start()
            }
            (
                HttpRange::Range(OrderedRange { start, end }),
                HttpContentRange::Bound(Bound { range, .. }),
            ) => start == range.start() && end == range.end(),
            (HttpRange::Suffix(suffix), HttpContentRange::Bound(Bound { range, .. })) => {
                (range.end() - range.start()).checked_add(1) == Some(suffix)
            }
            (
                HttpRange::StartingPoint(n),
                HttpContentRange::Unsatisfiable(Unsatisfiable { size }),
            )
            | (
                HttpRange::Range(OrderedRange { end: n, .. }),
                HttpContentRange::Unsatisfiable(Unsatisfiable { size }),
            ) => n >= *size,
            (
                HttpRange::Suffix(suffix),
                HttpContentRange::Unsatisfiable(Unsatisfiable { size }),
            ) => suffix > *size,
        }
    }
}

impl FromStr for HttpContentRange {
    type Err = ParseHttpRangeOrContentRangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseHttpRangeOrContentRangeError::Empty);
        }

        let (unit_str, range_and_size_str) = s
            .split_once(" ")
            .ok_or(ParseHttpRangeOrContentRangeError::Malformed)?;

        if unit_str != UNIT {
            return Err(ParseHttpRangeOrContentRangeError::InvalidUnit);
        }

        let (range_str, size_str) = range_and_size_str
            .split_once('/')
            .ok_or(ParseHttpRangeOrContentRangeError::Malformed)?;

        let range = range_str.parse::<ParsedRange>()?;
        let size = size_str
            .parse::<ParsedSize>()
            .map_err(ParseHttpRangeOrContentRangeError::InvalidSize)?;

        match (range, size) {
            (ParsedRange::Star, ParsedSize::Star) => {
                Err(ParseHttpRangeOrContentRangeError::Malformed)
            }
            (ParsedRange::Star, ParsedSize::Value(size)) => {
                Ok(Self::Unsatisfiable(Unsatisfiable { size }))
            }
            (ParsedRange::Range(range), ParsedSize::Star) => {
                Ok(Self::Bound(Bound { range, size: None }))
            }
            (ParsedRange::Range(range), ParsedSize::Value(size)) => Ok(Self::Bound(Bound {
                range,
                size: Some(size),
            })),
        }
    }
}

/// The Errors that may occur when creating a [`Bound`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBound {
    #[error(transparent)]
    InvalidRange(#[from] InvalidOrderedRange),
    #[error("The provided range `end`: {} is greater than or equal to `size`: {size}", range.end)]
    InvalidSize { range: OrderedRange, size: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bound {
    range: OrderedRange,
    size: Option<u64>,
}

impl Bound {
    // Creates a new [`Bound`].
    pub fn new(range: RangeInclusive<u64>, size: Option<u64>) -> Result<Self, InvalidBound> {
        let range = OrderedRange::new(range)?;

        if let Some(size) = size
            && range.end() >= size
        {
            return Err(InvalidBound::InvalidSize { range, size });
        }

        Ok(Self { range, size })
    }

    // Returns a copy of the [`Bound`] range.
    pub fn range(&self) -> OrderedRange {
        self.range
    }

    // Returns the size of the [`Bound`], if present.
    pub fn size(&self) -> Option<u64> {
        self.size
    }
}

// An unsatisfiable `Content-Range`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsatisfiable {
    size: u64,
}

impl Unsatisfiable {
    // Creates a new [`Unsatisfiable`].
    pub fn new(size: u64) -> Self {
        Self { size }
    }
}

#[derive(Debug, Clone, Copy)]
enum ParsedRange {
    Star,
    Range(OrderedRange),
}

impl FromStr for ParsedRange {
    type Err = ParseHttpRangeOrContentRangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "*" {
            return Ok(ParsedRange::Star);
        }

        let (start_str, end_str) = s
            .split_once('-')
            .ok_or(ParseHttpRangeOrContentRangeError::MalformedRange)?;

        let start = u64_unprefixed_parse(start_str)
            .map_err(ParseHttpRangeOrContentRangeError::InvalidRangePiece)?;
        let end = u64_unprefixed_parse(end_str)
            .map_err(ParseHttpRangeOrContentRangeError::InvalidRangePiece)?;

        let range = OrderedRange::new(start..=end)?;
        Ok(ParsedRange::Range(range))
    }
}

#[derive(Debug, Clone, Copy)]
enum ParsedSize {
    Star,
    Value(u64),
}

impl FromStr for ParsedSize {
    type Err = InvalidHttpU64;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == "*" {
            ParsedSize::Star
        } else {
            let size = u64_unprefixed_parse(s)?;
            ParsedSize::Value(size)
        })
    }
}

impl From<&HttpContentRange> for HeaderValue {
    fn from(value: &HttpContentRange) -> Self {
        HeaderValue::from_maybe_shared(value.to_string()).expect(
            "The `HttpContentRange` Display implementation produces nonvisible ASCII characters",
        )
    }
}

impl TryFrom<&HeaderValue> for HttpContentRange {
    type Error = ParseHttpRangeOrContentRangeError;
    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        value
            .to_str()
            .map_err(|_| ParseHttpRangeOrContentRangeError::ContainsNonVisibleASCII)?
            .parse::<Self>()
    }
}

#[cfg(feature = "axum")]
impl<S> axum_core::extract::OptionalFromRequestParts<S> for HttpContentRange
where
    S: Send + Sync,
{
    type Rejection = ParseHttpRangeOrContentRangeError;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match parts.headers.get(http::header::CONTENT_RANGE) {
            Some(content_range) => {
                let content_range = HttpContentRange::try_from(content_range)?;
                Ok(Some(content_range))
            }
            None => Ok(None),
        }
    }
}

impl Display for HttpContentRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpContentRange::Bound(Bound { range, size }) => match size {
                Some(size) => write!(f, "{UNIT} {range}/{size}"),
                None => write!(f, "{UNIT} {range}/*"),
            },
            HttpContentRange::Unsatisfiable(Unsatisfiable { size }) => write!(f, "{UNIT} */{size}"),
        }
    }
}
