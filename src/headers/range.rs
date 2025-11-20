use std::{
    fmt::{self, Display},
    str::FromStr,
};

use http::HeaderValue;

use crate::headers::{OrderedRange, ParseHttpRangeOrContentRangeError, UNIT, u64_unprefixed_parse};

/// A typed HTTP `Range` header that only supports a __single__ range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpRange {
    StartingPoint(u64),
    Range(OrderedRange),
    Suffix(u64),
}

impl FromStr for HttpRange {
    type Err = ParseHttpRangeOrContentRangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseHttpRangeOrContentRangeError::Empty);
        }

        let (unit_str, range_str) = s
            .split_once("=")
            .ok_or(ParseHttpRangeOrContentRangeError::Malformed)?;
        if unit_str != UNIT {
            return Err(ParseHttpRangeOrContentRangeError::InvalidUnit);
        }

        let (start_str, end_str) = range_str
            .split_once("-")
            .ok_or(ParseHttpRangeOrContentRangeError::MalformedRange)?;

        match (start_str.is_empty(), end_str.is_empty()) {
            (false, false) => {
                let start = u64_unprefixed_parse(start_str)
                    .map_err(ParseHttpRangeOrContentRangeError::InvalidRangePiece)?;
                let end = u64_unprefixed_parse(end_str)
                    .map_err(ParseHttpRangeOrContentRangeError::InvalidRangePiece)?;

                let range = OrderedRange::new(start..=end)?;
                Ok(Self::Range(range))
            }
            (false, true) => {
                let start = start_str
                    .parse()
                    .map_err(|_| ParseHttpRangeOrContentRangeError::MalformedRange)?;

                Ok(Self::StartingPoint(start))
            }
            (true, false) => {
                let suffix = end_str
                    .parse()
                    .map_err(|_| ParseHttpRangeOrContentRangeError::MalformedRange)?;

                Ok(Self::Suffix(suffix))
            }
            (true, true) => Err(ParseHttpRangeOrContentRangeError::Malformed),
        }
    }
}

impl From<&HttpRange> for HeaderValue {
    fn from(value: &HttpRange) -> Self {
        HeaderValue::from_maybe_shared(value.to_string())
            .expect("The `HttpRange` Display implementation produces nonvisible ASCII characters")
    }
}

impl TryFrom<&HeaderValue> for HttpRange {
    type Error = ParseHttpRangeOrContentRangeError;
    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        value
            .to_str()
            .map_err(|_| ParseHttpRangeOrContentRangeError::ContainsNonVisibleASCII)?
            .parse::<Self>()
    }
}

#[cfg(feature = "axum")]
impl<S> axum_core::extract::OptionalFromRequestParts<S> for HttpRange
where
    S: Send + Sync,
{
    type Rejection = ParseHttpRangeOrContentRangeError;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match parts.headers.get(http::header::RANGE) {
            Some(range) => {
                let range = HttpRange::try_from(range)?;
                Ok(Some(range))
            }
            None => Ok(None),
        }
    }
}

impl Display for HttpRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpRange::StartingPoint(start) => write!(f, "{UNIT}={start}-"),
            HttpRange::Range(range) => write!(f, "{UNIT}={range}"),
            HttpRange::Suffix(suffix) => write!(f, "{UNIT}=-{suffix}"),
        }
    }
}
