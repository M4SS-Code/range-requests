#[cfg(test)]
use crate::headers::OrderedRange;

#[test]
fn succesful_ordered_range() {
    assert!(OrderedRange::new(10..=11).is_ok())
}

#[test]
fn unsuccesful_ordered_range() {
    assert!(OrderedRange::new(11..=10).is_err())
}

#[cfg(test)]
mod content_range {
    use crate::headers::{
        InvalidOrderedRange, OrderedRange,
        content_range::{Bound, HttpContentRange, InvalidBound, Unsatisfiable},
    };

    #[test]
    fn succesful_bound() {
        assert!(Bound::new(10..=20, Some(50)).is_ok());
        assert!(Bound::new(10..=20, None).is_ok());
    }

    #[test]
    fn unsuccesful_bound() {
        assert_eq!(
            Bound::new(11..=10, None),
            Err(InvalidBound::InvalidRange(InvalidOrderedRange {
                start: 11,
                end: 10
            }))
        );
        assert_eq!(
            Bound::new(11..=10, Some(50)),
            Err(InvalidBound::InvalidRange(InvalidOrderedRange {
                start: 11,
                end: 10
            }))
        );

        assert_eq!(
            Bound::new(10..=50, Some(20)),
            Err(InvalidBound::InvalidSize {
                range: OrderedRange::new(10..=50).unwrap(),
                size: 20
            })
        );
    }

    #[test]
    fn succesful_sized_bound_parsing() {
        assert_eq!(
            "bytes 10-20/50".parse::<HttpContentRange>().unwrap(),
            HttpContentRange::Bound(Bound::new(10..=20, Some(50)).unwrap())
        );
    }

    #[test]
    fn succesful_sized_bound_to_string() {
        assert_eq!(
            "bytes 10-20/50",
            &HttpContentRange::Bound(Bound::new(10..=20, Some(50)).unwrap()).to_string()
        );
    }

    #[test]
    fn succesful_unsized_bound_parsing() {
        assert_eq!(
            "bytes 10-20/*".parse::<HttpContentRange>().unwrap(),
            HttpContentRange::Bound(Bound::new(10..=20, None).unwrap())
        );
    }

    #[test]
    fn succesful_unsized_bound_to_string() {
        assert_eq!(
            "bytes 10-20/*",
            &HttpContentRange::Bound(Bound::new(10..=20, None).unwrap()).to_string()
        );
    }

    #[test]
    fn succesful_unsatisfiable_parsing() {
        assert_eq!(
            "bytes */50".parse::<HttpContentRange>().unwrap(),
            HttpContentRange::Unsatisfiable(Unsatisfiable::new(50))
        );
    }

    #[test]
    fn succesful_unsatisfiable_to_string() {
        assert_eq!(
            "bytes */50",
            &HttpContentRange::Unsatisfiable(Unsatisfiable::new(50)).to_string()
        );
    }

    mod expected_range {
        use crate::headers::{
            OrderedRange,
            content_range::{Bound, HttpContentRange, Unsatisfiable},
            range::HttpRange,
        };

        #[test]
        fn sucessful_range_range_range_content_bound() {
            let range = HttpRange::Range(OrderedRange::new(10..=20).unwrap());
            let content_range = HttpContentRange::Bound(Bound::new(10..=20, Some(50)).unwrap());

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn unsucessful_range_range_range_content_bound() {
            let range = HttpRange::Range(OrderedRange::new(10..=50).unwrap());
            let content_range = HttpContentRange::Unsatisfiable(Unsatisfiable::new(20));

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn sucessful_range_starting_point_range_content_bound() {
            let range = HttpRange::StartingPoint(10);
            let content_range = HttpContentRange::Bound(Bound::new(10..=49, Some(50)).unwrap());

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn unsucessful_range_starting_point_range_content_bound() {
            let range = HttpRange::StartingPoint(21);
            let content_range = HttpContentRange::Unsatisfiable(Unsatisfiable::new(20));

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn sucessful_range_suffix_range_content_bound() {
            let range = HttpRange::Suffix(20);
            let content_range = HttpContentRange::Bound(Bound::new(0..=19, Some(20)).unwrap());

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn unsucessful_range_suffix_range_content_bound() {
            let range = HttpRange::Suffix(50);
            let content_range = HttpContentRange::Unsatisfiable(Unsatisfiable::new(20));

            assert!(content_range.matches_requested_range(range));
        }
    }
}

#[cfg(test)]
mod range {
    use crate::headers::{OrderedRange, ParseHttpRangeOrContentRangeError, range::HttpRange};

    #[test]
    fn succesful_starting_parsing() {
        assert_eq!(
            "bytes=50-".parse::<HttpRange>().unwrap(),
            HttpRange::StartingPoint(50)
        );
    }

    #[test]
    fn succesful_starting_to_string() {
        assert_eq!("bytes=50-", &HttpRange::StartingPoint(50).to_string());
    }

    #[test]
    fn succesful_range_parsing() {
        assert_eq!(
            "bytes=50-100".parse::<HttpRange>().unwrap(),
            HttpRange::Range(OrderedRange::new(50..=100).unwrap())
        );
    }

    #[test]
    fn succesful_range_to_string() {
        assert_eq!(
            "bytes=50-100",
            &HttpRange::Range(OrderedRange::new(50..=100).unwrap()).to_string()
        );
    }

    #[test]
    fn succesful_suffix_parsing() {
        assert_eq!(
            "bytes=-100".parse::<HttpRange>().unwrap(),
            HttpRange::Suffix(100)
        );
    }

    #[test]
    fn succesful_suffix_to_string() {
        assert_eq!("bytes=-100", &HttpRange::Suffix(100).to_string());
    }

    #[test]
    fn empty_input() {
        assert_eq!(
            "".parse::<HttpRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::Empty
        );
    }

    #[test]
    fn whitespace_only_input() {
        assert_eq!(
            "   ".parse::<HttpRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::Empty
        );
    }

    #[test]
    fn missing_equals() {
        assert_eq!(
            "bytes50-100".parse::<HttpRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::Malformed
        );
    }

    #[test]
    fn wrong_unit() {
        assert_eq!(
            "items=0-10".parse::<HttpRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::InvalidUnit
        );
    }

    #[test]
    fn both_empty() {
        assert_eq!(
            "bytes=-".parse::<HttpRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::Malformed
        );
    }

    #[test]
    fn unordered_range() {
        assert!("bytes=100-50".parse::<HttpRange>().is_err());
    }

    #[test]
    fn plus_prefix_rejected_in_starting_point() {
        assert!("bytes=+50-".parse::<HttpRange>().is_err());
    }

    #[test]
    fn plus_prefix_rejected_in_suffix() {
        assert!("bytes=-+100".parse::<HttpRange>().is_err());
    }

    #[test]
    fn plus_prefix_rejected_in_range() {
        assert!("bytes=+50-100".parse::<HttpRange>().is_err());
        assert!("bytes=50-+100".parse::<HttpRange>().is_err());
    }

    #[test]
    fn multi_range_rejected() {
        assert!("bytes=0-50, 100-150".parse::<HttpRange>().is_err());
    }

    #[test]
    fn suffix_zero_parsing() {
        assert_eq!(
            "bytes=-0".parse::<HttpRange>().unwrap(),
            HttpRange::Suffix(0)
        );
    }

    #[test]
    fn starting_point_zero() {
        assert_eq!(
            "bytes=0-".parse::<HttpRange>().unwrap(),
            HttpRange::StartingPoint(0)
        );
    }

    #[test]
    fn range_single_byte() {
        assert_eq!(
            "bytes=0-0".parse::<HttpRange>().unwrap(),
            HttpRange::Range(OrderedRange::new(0..=0).unwrap())
        );
    }
}

#[cfg(test)]
mod content_range_parsing_errors {
    use crate::headers::{ParseHttpRangeOrContentRangeError, content_range::HttpContentRange};

    #[test]
    fn empty_input() {
        assert_eq!(
            "".parse::<HttpContentRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::Empty
        );
    }

    #[test]
    fn missing_space() {
        assert_eq!(
            "bytes10-20/50".parse::<HttpContentRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::Malformed
        );
    }

    #[test]
    fn wrong_unit() {
        assert_eq!(
            "items 10-20/50".parse::<HttpContentRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::InvalidUnit
        );
    }

    #[test]
    fn missing_slash() {
        assert_eq!(
            "bytes 10-20".parse::<HttpContentRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::Malformed
        );
    }

    #[test]
    fn star_star() {
        assert_eq!(
            "bytes */*".parse::<HttpContentRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::Malformed
        );
    }

    #[test]
    fn end_exceeds_size() {
        assert_eq!(
            "bytes 10-20/15".parse::<HttpContentRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::MalformedRange
        );
    }

    #[test]
    fn end_equals_size() {
        assert_eq!(
            "bytes 0-20/20".parse::<HttpContentRange>().unwrap_err(),
            ParseHttpRangeOrContentRangeError::MalformedRange
        );
    }

    #[test]
    fn end_at_boundary() {
        assert!("bytes 0-19/20".parse::<HttpContentRange>().is_ok());
    }
}

#[cfg(test)]
mod file_range {
    use std::num::NonZeroU64;

    use crate::{
        file_range,
        headers::{
            OrderedRange,
            content_range::{Bound, HttpContentRange},
            range::HttpRange,
        },
    };

    fn size(n: u64) -> NonZeroU64 {
        NonZeroU64::new(n).unwrap()
    }

    #[test]
    fn no_range_returns_full_file() {
        let result = file_range(size(10), None).unwrap();
        assert!(result.header().is_none());
        assert_eq!(result.range(), &(0..=9));
    }

    #[test]
    fn starting_point_zero() {
        let result = file_range(size(10), Some(HttpRange::StartingPoint(0))).unwrap();
        assert!(result.header().is_some());
        assert_eq!(result.range(), &(0..=9));
    }

    #[test]
    fn starting_point_middle() {
        let result = file_range(size(10), Some(HttpRange::StartingPoint(5))).unwrap();
        assert_eq!(result.range(), &(5..=9));
    }

    #[test]
    fn starting_point_last_byte() {
        let result = file_range(size(10), Some(HttpRange::StartingPoint(9))).unwrap();
        assert_eq!(result.range(), &(9..=9));
    }

    #[test]
    fn starting_point_at_size() {
        let result = file_range(size(10), Some(HttpRange::StartingPoint(10)));
        assert!(result.is_err());
    }

    #[test]
    fn starting_point_beyond_size() {
        let result = file_range(size(10), Some(HttpRange::StartingPoint(20)));
        assert!(result.is_err());
    }

    #[test]
    fn range_single_byte() {
        let range = HttpRange::Range(OrderedRange::new(0..=0).unwrap());
        let result = file_range(size(10), Some(range)).unwrap();
        assert_eq!(result.range(), &(0..=0));
    }

    #[test]
    fn range_full_file() {
        let range = HttpRange::Range(OrderedRange::new(0..=9).unwrap());
        let result = file_range(size(10), Some(range)).unwrap();
        assert_eq!(result.range(), &(0..=9));
    }

    #[test]
    fn range_end_at_size() {
        let range = HttpRange::Range(OrderedRange::new(0..=10).unwrap());
        let result = file_range(size(10), Some(range));
        assert!(result.is_err());
    }

    #[test]
    fn range_beyond_size() {
        let range = HttpRange::Range(OrderedRange::new(0..=50).unwrap());
        let result = file_range(size(10), Some(range));
        assert!(result.is_err());
    }

    #[test]
    fn suffix_entire_file() {
        let result = file_range(size(10), Some(HttpRange::Suffix(10))).unwrap();
        assert_eq!(result.range(), &(0..=9));
    }

    #[test]
    fn suffix_last_byte() {
        let result = file_range(size(10), Some(HttpRange::Suffix(1))).unwrap();
        assert_eq!(result.range(), &(9..=9));
    }

    #[test]
    fn suffix_middle() {
        let result = file_range(size(10), Some(HttpRange::Suffix(5))).unwrap();
        assert_eq!(result.range(), &(5..=9));
    }

    #[test]
    fn suffix_zero_is_unsatisfiable() {
        let result = file_range(size(10), Some(HttpRange::Suffix(0)));
        assert!(result.is_err());
    }

    #[test]
    fn suffix_exceeds_size() {
        let result = file_range(size(10), Some(HttpRange::Suffix(11)));
        assert!(result.is_err());
    }

    #[test]
    fn size_one_no_range() {
        let result = file_range(size(1), None).unwrap();
        assert_eq!(result.range(), &(0..=0));
    }

    #[test]
    fn size_one_starting_point_zero() {
        let result = file_range(size(1), Some(HttpRange::StartingPoint(0))).unwrap();
        assert_eq!(result.range(), &(0..=0));
    }

    #[test]
    fn size_one_suffix_one() {
        let result = file_range(size(1), Some(HttpRange::Suffix(1))).unwrap();
        assert_eq!(result.range(), &(0..=0));
    }

    #[test]
    fn content_range_header_present_for_range_request() {
        let result = file_range(size(10), Some(HttpRange::StartingPoint(0))).unwrap();
        let header = result.header().unwrap();
        assert_eq!(
            header,
            HttpContentRange::Bound(Bound::new(0..=9, Some(10)).unwrap())
        );
    }
}

#[cfg(test)]
mod serve_file {
    use bytes::Bytes;

    use crate::{headers::range::HttpRange, serve_file_with_http_range};

    #[test]
    fn no_range_returns_full_body() {
        let body = Bytes::from_static(b"hello world");
        let result = serve_file_with_http_range(body.clone(), None).unwrap();
        assert_eq!(result.body(), &body);
        assert!(result.header().is_none());
    }

    #[test]
    fn range_slices_body() {
        let body = Bytes::from_static(b"hello world");
        let range = Some(HttpRange::StartingPoint(6));
        let result = serve_file_with_http_range(body, range).unwrap();
        assert_eq!(result.body(), &Bytes::from_static(b"world"));
        assert!(result.header().is_some());
    }

    #[test]
    fn suffix_slices_from_end() {
        let body = Bytes::from_static(b"hello world");
        let range = Some(HttpRange::Suffix(5));
        let result = serve_file_with_http_range(body, range).unwrap();
        assert_eq!(result.body(), &Bytes::from_static(b"world"));
    }

    #[test]
    fn empty_body_is_unsatisfiable() {
        let body = Bytes::new();
        let result = serve_file_with_http_range(body, None);
        assert!(result.is_err());
    }

    #[test]
    fn suffix_zero_is_unsatisfiable() {
        let body = Bytes::from_static(b"hello");
        let result = serve_file_with_http_range(body, Some(HttpRange::Suffix(0)));
        assert!(result.is_err());
    }
}
