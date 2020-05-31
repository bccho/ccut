/// Implements the cut function per line: this will split `line` by commas (taking both single and
/// double quotes into account) and return a string consisting of only the fields indicated by the
/// column indices specified. Absorbs out-of-bounds errors to handle ragged edge CSVs.
pub fn cut_line(line: &String, cols: &Vec<usize>) -> String {
    // Idea: do two passes - the first time to parse and the second time to produce the output.
    // TODO: terminate first pass early if we reached the max field?

    // Step 1: parse the fields. We don't use String::split() because we want to escape quotes.
    let fields = split_line(&line);

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

/// Splits a string `line` on commas, with double and single quotes accounted for
pub fn split_line(line: &String) -> Vec<&str> {
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

#[cfg(test)]
mod test_split_line {
    use super::*;

    #[test]
    fn test_basic() {
        let input = &String::from("a,b,c");
        let res = split_line(input);
        assert_eq!(res, vec!["a", "b", "c"]);
    }
    #[test]
    fn test_whitespace() {
        let input = &String::from(" a,b,c  ");
        let res = split_line(input);
        assert_eq!(res, vec!["a", "b", "c"]);
    }
    #[test]
    fn test_double_quote() {
        let input = &String::from(r#"a,"b,c""#);
        let res = split_line(input);
        assert_eq!(res, vec!["a", "\"b,c\""]);
    }
    #[test]
    fn test_single_quote() {
        let input = &String::from(r#"a,'b,c'"#);
        let res = split_line(input);
        assert_eq!(res, vec!["a", "\'b,c\'"]);
    }
}