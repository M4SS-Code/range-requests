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
    use crate::headers::{OrderedRange, range::HttpRange};

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
}
