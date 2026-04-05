#[cfg(test)]
use crate::headers::OrderedRange;

#[test]
fn successful_ordered_range() {
    assert!(OrderedRange::new(10..=11).is_ok())
}

#[test]
fn unsuccessful_ordered_range() {
    assert!(OrderedRange::new(11..=10).is_err())
}

#[cfg(test)]
mod content_range {
    use crate::headers::{
        InvalidOrderedRange, OrderedRange,
        content_range::{Bound, HttpContentRange, InvalidBound, Unsatisfiable},
    };

    #[test]
    fn successful_bound() {
        assert!(Bound::new(10..=20, Some(50)).is_ok());
        assert!(Bound::new(10..=20, None).is_ok());
    }

    #[test]
    fn unsuccessful_bound() {
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
    fn successful_sized_bound_parsing() {
        assert_eq!(
            "bytes 10-20/50".parse::<HttpContentRange>().unwrap(),
            HttpContentRange::Bound(Bound::new(10..=20, Some(50)).unwrap())
        );
    }

    #[test]
    fn successful_sized_bound_to_string() {
        assert_eq!(
            "bytes 10-20/50",
            &HttpContentRange::Bound(Bound::new(10..=20, Some(50)).unwrap()).to_string()
        );
    }

    #[test]
    fn successful_unsized_bound_parsing() {
        assert_eq!(
            "bytes 10-20/*".parse::<HttpContentRange>().unwrap(),
            HttpContentRange::Bound(Bound::new(10..=20, None).unwrap())
        );
    }

    #[test]
    fn successful_unsized_bound_to_string() {
        assert_eq!(
            "bytes 10-20/*",
            &HttpContentRange::Bound(Bound::new(10..=20, None).unwrap()).to_string()
        );
    }

    #[test]
    fn successful_unsatisfiable_parsing() {
        assert_eq!(
            "bytes */50".parse::<HttpContentRange>().unwrap(),
            HttpContentRange::Unsatisfiable(Unsatisfiable::new(50))
        );
    }

    #[test]
    fn successful_unsatisfiable_to_string() {
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
        fn successful_range_range_range_content_bound() {
            let range = HttpRange::Range(OrderedRange::new(10..=20).unwrap());
            let content_range = HttpContentRange::Bound(Bound::new(10..=20, Some(50)).unwrap());

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn unsuccessful_range_range_range_content_bound() {
            let range = HttpRange::Range(OrderedRange::new(10..=50).unwrap());
            let content_range = HttpContentRange::Unsatisfiable(Unsatisfiable::new(20));

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn successful_range_starting_point_range_content_bound() {
            let range = HttpRange::StartingPoint(10);
            let content_range = HttpContentRange::Bound(Bound::new(10..=49, Some(50)).unwrap());

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn unsuccessful_range_starting_point_range_content_bound() {
            let range = HttpRange::StartingPoint(21);
            let content_range = HttpContentRange::Unsatisfiable(Unsatisfiable::new(20));

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn successful_range_suffix_range_content_bound() {
            let range = HttpRange::Suffix(20);
            let content_range = HttpContentRange::Bound(Bound::new(0..=19, Some(20)).unwrap());

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn unsuccessful_range_suffix_zero_content_unsatisfiable() {
            let range = HttpRange::Suffix(0);
            let content_range = HttpContentRange::Unsatisfiable(Unsatisfiable::new(20));

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn suffix_exceeding_size_is_not_unsatisfiable() {
            let range = HttpRange::Suffix(50);
            let content_range = HttpContentRange::Unsatisfiable(Unsatisfiable::new(20));

            assert!(!content_range.matches_requested_range(range));
        }

        #[test]
        fn range_with_clamped_end_matches() {
            let range = HttpRange::Range(OrderedRange::new(0..=999).unwrap());
            let content_range = HttpContentRange::Bound(Bound::new(0..=49, Some(50)).unwrap());

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn suffix_clamped_to_file_size_matches() {
            let range = HttpRange::Suffix(999);
            let content_range = HttpContentRange::Bound(Bound::new(0..=49, Some(50)).unwrap());

            assert!(content_range.matches_requested_range(range));
        }

        #[test]
        fn suffix_not_at_boundary_does_not_match() {
            let range = HttpRange::Suffix(5);
            let content_range = HttpContentRange::Bound(Bound::new(0..=4, Some(20)).unwrap());

            assert!(!content_range.matches_requested_range(range));
        }

        #[test]
        fn suffix_length_mismatch_does_not_match() {
            let range = HttpRange::Suffix(3);
            let content_range = HttpContentRange::Bound(Bound::new(10..=19, Some(20)).unwrap());

            assert!(!content_range.matches_requested_range(range));
        }
    }
}

#[cfg(test)]
mod range {
    use crate::headers::{OrderedRange, ParseHttpRangeOrContentRangeError, range::HttpRange};

    #[test]
    fn successful_starting_parsing() {
        assert_eq!(
            "bytes=50-".parse::<HttpRange>().unwrap(),
            HttpRange::StartingPoint(50)
        );
    }

    #[test]
    fn successful_starting_to_string() {
        assert_eq!("bytes=50-", &HttpRange::StartingPoint(50).to_string());
    }

    #[test]
    fn successful_range_parsing() {
        assert_eq!(
            "bytes=50-100".parse::<HttpRange>().unwrap(),
            HttpRange::Range(OrderedRange::new(50..=100).unwrap())
        );
    }

    #[test]
    fn successful_range_to_string() {
        assert_eq!(
            "bytes=50-100",
            &HttpRange::Range(OrderedRange::new(50..=100).unwrap()).to_string()
        );
    }

    #[test]
    fn successful_suffix_parsing() {
        assert_eq!(
            "bytes=-100".parse::<HttpRange>().unwrap(),
            HttpRange::Suffix(100)
        );
    }

    #[test]
    fn successful_suffix_to_string() {
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
    use crate::{
        file_range,
        headers::{
            OrderedRange,
            content_range::{Bound, HttpContentRange},
            range::HttpRange,
        },
    };

    #[test]
    fn no_range_returns_full_file() {
        let result = file_range(10, None).unwrap();
        assert!(result.header().is_none());
        assert_eq!(result.range(), &(0..10));
    }

    #[test]
    fn starting_point_zero() {
        let result = file_range(10, Some(HttpRange::StartingPoint(0))).unwrap();
        assert!(result.header().is_some());
        assert_eq!(result.range(), &(0..10));
    }

    #[test]
    fn starting_point_middle() {
        let result = file_range(10, Some(HttpRange::StartingPoint(5))).unwrap();
        assert_eq!(result.range(), &(5..10));
    }

    #[test]
    fn starting_point_last_byte() {
        let result = file_range(10, Some(HttpRange::StartingPoint(9))).unwrap();
        assert_eq!(result.range(), &(9..10));
    }

    #[test]
    fn starting_point_at_size() {
        let result = file_range(10, Some(HttpRange::StartingPoint(10)));
        assert!(result.is_err());
    }

    #[test]
    fn starting_point_beyond_size() {
        let result = file_range(10, Some(HttpRange::StartingPoint(20)));
        assert!(result.is_err());
    }

    #[test]
    fn range_single_byte() {
        let range = HttpRange::Range(OrderedRange::new(0..=0).unwrap());
        let result = file_range(10, Some(range)).unwrap();
        assert_eq!(result.range(), &(0..1));
    }

    #[test]
    fn range_full_file() {
        let range = HttpRange::Range(OrderedRange::new(0..=9).unwrap());
        let result = file_range(10, Some(range)).unwrap();
        assert_eq!(result.range(), &(0..10));
    }

    #[test]
    fn range_end_at_size_is_clamped() {
        let range = HttpRange::Range(OrderedRange::new(0..=10).unwrap());
        let result = file_range(10, Some(range)).unwrap();
        assert_eq!(result.range(), &(0..10));
    }

    #[test]
    fn range_beyond_size_is_clamped() {
        let range = HttpRange::Range(OrderedRange::new(0..=50).unwrap());
        let result = file_range(10, Some(range)).unwrap();
        assert_eq!(result.range(), &(0..10));
    }

    #[test]
    fn range_end_at_u64_max_is_clamped() {
        let range = HttpRange::Range(OrderedRange::new(0..=u64::MAX).unwrap());
        let result = file_range(10, Some(range)).unwrap();
        assert_eq!(result.range(), &(0..10));
    }

    #[test]
    fn range_start_at_size_is_unsatisfiable() {
        let range = HttpRange::Range(OrderedRange::new(10..=20).unwrap());
        let result = file_range(10, Some(range));
        assert!(result.is_err());
    }

    #[test]
    fn range_start_beyond_size_is_unsatisfiable() {
        let range = HttpRange::Range(OrderedRange::new(50..=100).unwrap());
        let result = file_range(10, Some(range));
        assert!(result.is_err());
    }

    #[test]
    fn suffix_entire_file() {
        let result = file_range(10, Some(HttpRange::Suffix(10))).unwrap();
        assert_eq!(result.range(), &(0..10));
    }

    #[test]
    fn suffix_last_byte() {
        let result = file_range(10, Some(HttpRange::Suffix(1))).unwrap();
        assert_eq!(result.range(), &(9..10));
    }

    #[test]
    fn suffix_middle() {
        let result = file_range(10, Some(HttpRange::Suffix(5))).unwrap();
        assert_eq!(result.range(), &(5..10));
    }

    #[test]
    fn suffix_zero_is_unsatisfiable() {
        let result = file_range(10, Some(HttpRange::Suffix(0)));
        assert!(result.is_err());
    }

    #[test]
    fn suffix_exceeds_size_is_clamped() {
        let result = file_range(10, Some(HttpRange::Suffix(11))).unwrap();
        assert_eq!(result.range(), &(0..10));
    }

    #[test]
    fn size_one_no_range() {
        let result = file_range(1, None).unwrap();
        assert_eq!(result.range(), &(0..1));
    }

    #[test]
    fn size_one_starting_point_zero() {
        let result = file_range(1, Some(HttpRange::StartingPoint(0))).unwrap();
        assert_eq!(result.range(), &(0..1));
    }

    #[test]
    fn size_one_suffix_one() {
        let result = file_range(1, Some(HttpRange::Suffix(1))).unwrap();
        assert_eq!(result.range(), &(0..1));
    }

    #[test]
    fn content_range_header_present_for_range_request() {
        let result = file_range(10, Some(HttpRange::StartingPoint(0))).unwrap();
        let header = result.header().unwrap();
        assert_eq!(
            header,
            HttpContentRange::Bound(Bound::new(0..=9, Some(10)).unwrap())
        );
    }

    #[test]
    fn size_zero_no_range() {
        let result = file_range(0, None).unwrap();
        assert!(result.header().is_none());
        assert_eq!(result.range(), &(0..0));
    }

    #[test]
    fn size_zero_starting_point_is_unsatisfiable() {
        let result = file_range(0, Some(HttpRange::StartingPoint(0)));
        assert!(result.is_err());
    }

    #[test]
    fn size_zero_range_is_unsatisfiable() {
        let range = HttpRange::Range(OrderedRange::new(0..=0).unwrap());
        let result = file_range(0, Some(range));
        assert!(result.is_err());
    }

    #[test]
    fn size_zero_suffix_is_unsatisfiable() {
        let result = file_range(0, Some(HttpRange::Suffix(1)));
        assert!(result.is_err());
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
    fn empty_body_returns_empty_response() {
        let body = Bytes::new();
        let result = serve_file_with_http_range(body, None).unwrap();
        assert!(result.body().is_empty());
        assert!(result.header().is_none());
    }

    #[test]
    fn suffix_zero_is_unsatisfiable() {
        let body = Bytes::from_static(b"hello");
        let result = serve_file_with_http_range(body, Some(HttpRange::Suffix(0)));
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod if_range {
    use http::HeaderValue;

    use crate::headers::{if_range::IfRange, range::HttpRange};

    #[test]
    fn parse_date() {
        let ir: IfRange = "Sun, 05 Apr 2026 04:49:21 GMT".parse().unwrap();
        assert!(matches!(ir, IfRange::Date(_)));
    }

    #[test]
    fn parse_strong_etag() {
        let ir: IfRange = "\"abc123\"".parse().unwrap();
        assert!(matches!(ir, IfRange::ETag(_)));
    }

    #[test]
    fn parse_weak_etag() {
        let ir: IfRange = "W/\"abc123\"".parse().unwrap();
        assert!(matches!(ir, IfRange::ETag(_)));
    }

    #[test]
    fn empty_rejected() {
        assert!("".parse::<IfRange>().is_err());
    }

    #[test]
    fn whitespace_only_rejected() {
        assert!("   ".parse::<IfRange>().is_err());
    }

    #[test]
    fn date_matches_last_modified() {
        let ir: IfRange = "Sun, 05 Apr 2026 04:49:21 GMT".parse().unwrap();
        let lm = HeaderValue::from_static("Sun, 05 Apr 2026 04:49:21 GMT");
        let range = HttpRange::StartingPoint(0);

        assert_eq!(ir.evaluate(range, Some(&lm), None), Some(range));
    }

    #[test]
    fn date_does_not_match_different_last_modified() {
        let ir: IfRange = "Mon, 01 Jan 2024 00:00:00 GMT".parse().unwrap();
        let lm = HeaderValue::from_static("Sun, 05 Apr 2026 04:49:21 GMT");
        let range = HttpRange::StartingPoint(0);

        assert_eq!(ir.evaluate(range, Some(&lm), None), None);
    }

    #[test]
    fn date_does_not_match_missing_last_modified() {
        let ir: IfRange = "Sun, 05 Apr 2026 04:49:21 GMT".parse().unwrap();
        let range = HttpRange::StartingPoint(0);

        assert_eq!(ir.evaluate(range, None, None), None);
    }

    #[test]
    fn strong_etag_matches() {
        let ir: IfRange = "\"abc123\"".parse().unwrap();
        let etag = HeaderValue::from_static("\"abc123\"");
        let range = HttpRange::StartingPoint(0);

        assert_eq!(ir.evaluate(range, None, Some(&etag)), Some(range));
    }

    #[test]
    fn strong_etag_does_not_match_different() {
        let ir: IfRange = "\"abc123\"".parse().unwrap();
        let etag = HeaderValue::from_static("\"xyz789\"");
        let range = HttpRange::StartingPoint(0);

        assert_eq!(ir.evaluate(range, None, Some(&etag)), None);
    }

    #[test]
    fn strong_etag_does_not_match_missing() {
        let ir: IfRange = "\"abc123\"".parse().unwrap();
        let range = HttpRange::StartingPoint(0);

        assert_eq!(ir.evaluate(range, None, None), None);
    }

    #[test]
    fn weak_etag_never_matches_in_strong_comparison() {
        let ir: IfRange = "W/\"abc123\"".parse().unwrap();
        let etag = HeaderValue::from_static("W/\"abc123\"");
        let range = HttpRange::StartingPoint(0);

        assert_eq!(ir.evaluate(range, None, Some(&etag)), None);
    }

    #[test]
    fn weak_if_range_does_not_match_strong_etag() {
        let ir: IfRange = "W/\"abc123\"".parse().unwrap();
        let etag = HeaderValue::from_static("\"abc123\"");
        let range = HttpRange::StartingPoint(0);

        assert_eq!(ir.evaluate(range, None, Some(&etag)), None);
    }

    #[test]
    fn strong_if_range_does_not_match_weak_etag() {
        let ir: IfRange = "\"abc123\"".parse().unwrap();
        let etag = HeaderValue::from_static("W/\"abc123\"");
        let range = HttpRange::StartingPoint(0);

        assert_eq!(ir.evaluate(range, None, Some(&etag)), None);
    }

    #[test]
    fn date_ignores_etag() {
        let ir: IfRange = "Sun, 05 Apr 2026 04:49:21 GMT".parse().unwrap();
        let etag = HeaderValue::from_static("\"abc123\"");
        let range = HttpRange::StartingPoint(0);

        // Date-based If-Range only checks Last-Modified, not ETag
        assert_eq!(ir.evaluate(range, None, Some(&etag)), None);
    }

    #[test]
    fn etag_ignores_last_modified() {
        let ir: IfRange = "\"abc123\"".parse().unwrap();
        let lm = HeaderValue::from_static("Sun, 05 Apr 2026 04:49:21 GMT");
        let range = HttpRange::StartingPoint(0);

        // ETag-based If-Range only checks ETag, not Last-Modified
        assert_eq!(ir.evaluate(range, Some(&lm), None), None);
    }
}
