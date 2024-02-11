fn find_first_of(tested: impl Fn(/*data*/ &[u8], /*searched*/ &[u8]) -> Option<usize>) {
    let letters = b"abcdefghijklmnopqrstuvwxyz";
    let fillers = b"0123456789_-&~#>";

    for searched_size in 1..letters.len() {
        let searched = &letters[0..=searched_size];
        for prefix_size in 0..100 {
            let mut data: Vec<u8> = std::iter::repeat(fillers)
                .flat_map(|v| v.iter())
                .take(prefix_size)
                .cloned()
                .collect();
            assert!(tested(&data, searched).is_none());
            data.extend(searched);
            for rotated in 0..searched.len() {
                data[prefix_size..].rotate_right(rotated);
                assert_eq!(tested(&data, searched), Some(prefix_size));
            }
        }
    }
}

fn find_first_not_of(tested: impl Fn(/*data*/ &[u8], /*not_searched*/ &[u8]) -> Option<usize>) {
    let letters = b"abcdefghijklmnopqrstuvwxyz";
    let fillers = b"0123456789_-&~#>";

    for not_searched_size in 1..letters.len() {
        let not_searched = &letters[0..=not_searched_size];
        for prefix_size in 0..100 {
            let mut data: Vec<u8> = std::iter::repeat(&not_searched)
                .flat_map(|v| v.iter())
                .take(prefix_size)
                .cloned()
                .collect();
            assert!(tested(&data, not_searched).is_none());
            data.extend(fillers);
            for rotated in 0..fillers.len() {
                data[prefix_size..].rotate_right(rotated);
                assert_eq!(tested(&data, not_searched), Some(prefix_size));
            }
        }
    }
}

mod generic {
    #[test]
    fn find_first_of() {
        super::find_first_of(crate::generic::find_first_of)
    }

    #[test]
    fn find_first_not_of() {
        super::find_first_not_of(crate::generic::find_first_not_of)
    }
}

mod sse42 {
    #[test]
    fn find_first_of() {
        if is_x86_feature_detected!("sse4.2") {
            super::find_first_of(|data, searched| unsafe {
                crate::sse42::find_first_of(data, searched)
            })
        }
    }

    #[test]
    fn find_first_not_of() {
        if is_x86_feature_detected!("sse4.2") {
            super::find_first_not_of(|data, searched| unsafe {
                crate::sse42::find_first_not_of(data, searched)
            })
        }
    }
}
