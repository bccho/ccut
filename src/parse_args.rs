/// This function parses a string representing the indices of columns to output.  `offset` (either
/// 0 or 1) indicates the index of the first column, which also affects how ranges are interpreted.
/// The result vector always zero-indexes columns so we don't have to worry about this offset
/// business elsewhere.
pub fn parse_arg_cols(cols: &String, offset: usize) -> Vec<usize> {
    assert!(offset == 0 || offset == 1, "Invalid offset, {}", offset);

    let mut res = Vec::new();
    // Columns are either ints or ranges ("int-int") separated by commas
    for elem in cols.split(',') {
        let elem = elem.trim();
        if elem.contains("-") {
            // It's a range
            let rg: Vec<&str> = elem.split('-').collect();
            assert!(rg.len() == 2, "Invalid range {} ({} parts)", elem, rg.len());
            let a: usize = rg[0].parse()
                .expect("Invalid range: start index is not an integer");
            let b: usize = rg[1].parse()
                .expect("Invalid range: end index is not an integer");
            // Validation
            assert!(offset <= a, "Start index must be at least {}", offset);
            if offset == 0 {
                assert!(a < b, "Overlapping end-points [{}, {})", a, b);
            } else {
                assert!(a <= b, "Overlapping end-points [{}, {}]", a, b);
            }
            // Push all the indices in the range
            for i in a..b + offset {
                res.push(i - offset);
            }
        } else {
            // It's a single number
            let i: usize = elem.parse()
                .expect("Invalid index");
            res.push(i - offset);
        }
    }
    return res;
}

#[cfg(test)]
mod test_parse_cols {
    use super::*;

    #[test]
    fn test_parse_individual() {
        let res = parse_arg_cols(&String::from("1,2,3"), 1);
        assert_eq!(res, vec![0, 1, 2]);
        let res = parse_arg_cols(&String::from("1,2,3"), 0);
        assert_eq!(res, vec![1, 2, 3]);
    }
    #[test]
    fn test_parse_range() {
        let res = parse_arg_cols(&String::from("1-3"), 1);
        assert_eq!(res, vec![0, 1, 2]);
        let res = parse_arg_cols(&String::from("1-3"), 0);
        assert_eq!(res, vec![1, 2]);
        let res = parse_arg_cols(&String::from("2-3"), 0);
        assert_eq!(res, vec![2]);
        let res = parse_arg_cols(&String::from("2-2"), 1);
        assert_eq!(res, vec![1]);
    }
    #[test]
    fn test_parse_combination() {
        let res = parse_arg_cols(&String::from("1-3,5,7"), 1);
        assert_eq!(res, vec![0, 1, 2, 4, 6]);
        let res = parse_arg_cols(&String::from("0-5,1,2"), 0);
        assert_eq!(res, vec![0, 1, 2, 3, 4, 1, 2]);
        let res = parse_arg_cols(&String::from("5,1-3,0"), 0);
        assert_eq!(res, vec![5, 1, 2, 0]);
    }

    #[test]
    #[should_panic]
    fn test_multiple_range_fails() {
        parse_arg_cols(&String::from("0-1-2"), 0);
    }
    #[test]
    #[should_panic]
    fn test_overlap_range_fails_0() {
        parse_arg_cols(&String::from("2-2"), 0);
    }
    #[test]
    #[should_panic]
    fn test_overlap_range_fails_1() {
        parse_arg_cols(&String::from("2-1"), 0);
    }
    #[test]
    #[should_panic]
    fn test_overlap_range_fails_2() {
        parse_arg_cols(&String::from("2-1"), 1);
    }
    #[test]
    #[should_panic]
    fn test_bad_offset_fails() {
        parse_arg_cols(&String::from("5"), 2);
    }
    #[test]
    #[should_panic]
    fn test_offset_fails() {
        parse_arg_cols(&String::from("0"), 1);
    }
    #[test]
    #[should_panic]
    fn test_offset_range_fails() {
        parse_arg_cols(&String::from("0-5"), 1);
    }
    #[test]
    #[should_panic]
    fn test_bad_inds_fails_0() {
        parse_arg_cols(&String::from("-1-5"), 0);
    }
    #[test]
    #[should_panic]
    fn test_bad_inds_fails_1() {
        parse_arg_cols(&String::from("0-5."), 0);
    }
    #[test]
    #[should_panic]
    fn test_bad_inds_fails_2() {
        parse_arg_cols(&String::from("a-b"), 0);
    }
    #[test]
    #[should_panic]
    fn test_bad_inds_fails_3() {
        parse_arg_cols(&String::from("a"), 0);
    }
}
