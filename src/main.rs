use std::io;

extern crate argparse;
use argparse::{ArgumentParser, Store, StoreTrue, StoreConst};

mod line;
mod parse_args;

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
                let fields: Vec<&str> = line::split_line(&line);
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

    let cols = parse_args::parse_arg_cols(&cols, offset);

    let mut line = String::new();
    loop {
        match io::stdin().read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                let res = line::cut_line(&line, &cols);
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
