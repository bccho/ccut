use std::io;

extern crate argparse;

use argparse::{ArgumentParser, Store, StoreTrue, StoreConst};

/// This function parses a string representing the indices of columns to output.  `offset` (either
/// 0 or 1) indicates the index of the first column, which also affects how ranges are interpreted.
/// The result vector always zero-indexes columns so we don't have to worry about this offset
/// business elsewhere.
fn parse_cols(cols: &String, offset: usize) -> Vec<usize> {
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

fn _split_line(line: &String) -> Vec<&str> {
    let line = line.trim();

    enum QuoteState { Normal, SingleQuote, DoubleQuote, SingleEscape, DoubleEscape };

    let mut fields: Vec<&str> = Vec::new();
    let mut state = QuoteState::Normal;
    let mut field_start: usize = 0;
    for (i, c) in line.chars().enumerate() {
        match (&state, c) {
            (QuoteState::Normal, ',') => {
                // it's the end of a field - push it and start a new one
                fields.push(&line[field_start..i]);
                field_start = i + 1;
            },
            // state machine logic for quoting and escaping
            (QuoteState::Normal,        '\'')   => state = QuoteState::SingleQuote,
            (QuoteState::SingleQuote,   '\'')   => state = QuoteState::Normal,
            (QuoteState::Normal,        '"')    => state = QuoteState::DoubleQuote,
            (QuoteState::DoubleQuote,   '"')    => state = QuoteState::Normal,
            (QuoteState::SingleQuote,   '\\')   => state = QuoteState::SingleEscape,
            (QuoteState::SingleEscape,  _)      => state = QuoteState::SingleQuote,
            (QuoteState::DoubleQuote,   '\\')   => state = QuoteState::DoubleEscape,
            (QuoteState::DoubleEscape,  _)      => state = QuoteState::DoubleQuote,
            _ => {},
        }
    }
    fields.push(&line[field_start..]);
    fields
}

/// Implements the cut function per line: this will split `line` by commas (taking both single and
/// double quotes into account) and return a string consisting of only the fields indicated by the
/// column indices specified. Absorbs out-of-bounds errors to handle ragged edge CSVs.
fn cut_line(line: &String, cols: &Vec<usize>) -> String {
    // Idea: do two passes - the first time to parse and the second time to produce the output.
    // TODO: terminate first pass early if we reached the max field?

    // Step 1: parse the fields. We don't use String::split() because we want to escape quotes.
    let fields = _split_line(&line);

    // Step 2: stitch together the output
    let mut res: Vec<&str> = Vec::new();
    for i in cols.iter() {
        if *i >= fields.len() {
            res.push(&"");
        } else {
            res.push(fields[*i]);
        }
    }

    res.join(",")
}

#[cfg(test)]
mod test_parse_cols {
    use super::*;

    #[test]
    fn test_parse_individual() {
        let res = parse_cols(&String::from("1,2,3"), 1);
        assert_eq!(res, vec![0, 1, 2]);
        let res = parse_cols(&String::from("1,2,3"), 0);
        assert_eq!(res, vec![1, 2, 3]);
    }
    #[test]
    fn test_parse_range() {
        let res = parse_cols(&String::from("1-3"), 1);
        assert_eq!(res, vec![0, 1, 2]);
        let res = parse_cols(&String::from("1-3"), 0);
        assert_eq!(res, vec![1, 2]);
        let res = parse_cols(&String::from("2-3"), 0);
        assert_eq!(res, vec![2]);
        let res = parse_cols(&String::from("2-2"), 1);
        assert_eq!(res, vec![1]);
    }
    #[test]
    fn test_parse_combination() {
        let res = parse_cols(&String::from("1-3,5,7"), 1);
        assert_eq!(res, vec![0, 1, 2, 4, 6]);
        let res = parse_cols(&String::from("0-5,1,2"), 0);
        assert_eq!(res, vec![0, 1, 2, 3, 4, 1, 2]);
        let res = parse_cols(&String::from("5,1-3,0"), 0);
        assert_eq!(res, vec![5, 1, 2, 0]);
    }

    #[test]
    #[should_panic]
    fn test_multiple_range_fails() {
        parse_cols(&String::from("0-1-2"), 0);
    }
    #[test]
    #[should_panic]
    fn test_overlap_range_fails_0() {
        parse_cols(&String::from("2-2"), 0);
    }
    #[test]
    #[should_panic]
    fn test_overlap_range_fails_1() {
        parse_cols(&String::from("2-1"), 0);
    }
    #[test]
    #[should_panic]
    fn test_overlap_range_fails_2() {
        parse_cols(&String::from("2-1"), 1);
    }
    #[test]
    #[should_panic]
    fn test_bad_offset_fails() {
        parse_cols(&String::from("5"), 2);
    }
    #[test]
    #[should_panic]
    fn test_offset_fails() {
        parse_cols(&String::from("0"), 1);
    }
    #[test]
    #[should_panic]
    fn test_offset_range_fails() {
        parse_cols(&String::from("0-5"), 1);
    }
    #[test]
    #[should_panic]
    fn test_bad_inds_fails_0() {
        parse_cols(&String::from("-1-5"), 0);
    }
    #[test]
    #[should_panic]
    fn test_bad_inds_fails_1() {
        parse_cols(&String::from("0-5."), 0);
    }
    #[test]
    #[should_panic]
    fn test_bad_inds_fails_2() {
        parse_cols(&String::from("a-b"), 0);
    }
    #[test]
    #[should_panic]
    fn test_bad_inds_fails_3() {
        parse_cols(&String::from("a"), 0);
    }
}

#[cfg(test)]
mod test_cut_line {
    use super::*;

    #[test]
    fn test_basic() {
        let res = cut_line(&String::from("a,b,c,d,e,f"), &vec![0, 2, 4]);
        assert_eq!(res, String::from("a,c,e"));
        let res = cut_line(&String::from("a,b,c,d,e,f"), &vec![0, 2, 4, 1, 3]);
        assert_eq!(res, String::from("a,c,e,b,d"));
        let res = cut_line(&String::from("a,b,c,d,e,f"), &vec![0, 0, 2, 2]);
        assert_eq!(res, String::from("a,a,c,c"));
    }
    #[test]
    fn test_handle_oob() {
        let res = cut_line(&String::from("a,b,c,d,e,f"), &vec![0, 0, 2, 2, 100, 4, 4]);
        assert_eq!(res, String::from("a,a,c,c,,e,e"));
        let res = cut_line(&String::from("a,b,c"), &vec![0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(res, String::from("a,b,c,,,,"));
    }
    #[test]
    fn test_quotes() {
        let res = cut_line(&String::from(r#"a,"b",c"#), &vec![0, 1, 2]);
        assert_eq!(res, String::from(r#"a,"b",c"#));
        let res = cut_line(&String::from(r#"a,'b',c"#), &vec![0, 1, 2]);
        assert_eq!(res, String::from(r#"a,'b',c"#));
        let res = cut_line(&String::from(r#"a,'"b""',c"#), &vec![0, 1, 2]);
        assert_eq!(res, String::from(r#"a,'"b""',c"#));
        let res = cut_line(&String::from(r#"a,'b,b',c"#), &vec![0, 1, 2]);
        assert_eq!(res, String::from(r#"a,'b,b',c"#));
        let res = cut_line(&String::from(r#"a,'b,b",c"#), &vec![0, 1, 2]);
        assert_eq!(res, String::from(r#"a,'b,b",c,"#));
        let res = cut_line(&String::from(r#"a,'b\'\",b',c"#), &vec![0, 1, 2]);
        assert_eq!(res, String::from(r#"a,'b\'\",b',c"#));
        let res = cut_line(&String::from(r#"a,"b\\\",b",c"#), &vec![0, 1, 2]);
        assert_eq!(res, String::from(r#"a,"b\\\",b",c"#));
        let res = cut_line(&String::from(r#"c,"d,\'d,\",d",e,f",",'g,\',g',h"#), &vec![0, 3, 5]);
        assert_eq!(res, String::from(r#"c,f",",h"#));
    }
}

fn main() {
    // Parse arguments
    let mut preview = false;
    let mut cols = String::from("");
    let mut offset: usize = 1;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Like cut, but for CSVs");
        ap.refer(&mut preview)
            .add_option(&["-p", "--preview"],
                        StoreTrue,
                        "Preview first line with column numbers");
        ap.refer(&mut offset)
            .add_option(&["-0", "--zero"],
                        StoreConst(0),
                        "Zero-index columns. Ranges are half-open like [a, b)")
            .add_option(&["-1", "--one"],
                        StoreConst(1),
                        "One-index columns (default). Ranges are closed like [a, b]");
        ap.refer(&mut cols)
            .add_argument("cols", Store, "Column indices to print");
        ap.parse_args_or_exit();
    }

    if preview {
        // TODO: dedup
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    return;
                }
                let fields: Vec<&str> = _split_line(&line);
                let col_nums: Vec<String> = fields.iter().enumerate()
                    .map(|tpl| (tpl.0 + offset).to_string())
                    .collect();
                println!("{}", col_nums.join(","));
                println!("{}", fields.join(","));
            },
            Err(error) => {
                println!("Error while reading stdin: {}", error);
                return;
            },
        }
        return;
    }

    let cols = parse_cols(&cols, offset);

    let mut line = String::new();
    loop {
        match io::stdin().read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                let res = cut_line(&line, &cols);
                println!("{}", res);
            },
            Err(error) => {
                println!("Error while reading stdin: {}", error);
                break;
            },
        }
        line.clear();
    }
}
