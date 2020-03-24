use std::io;
use std::io::prelude::*;
use std::collections::HashMap;

use regex::Regex;

#[macro_use]
extern crate lazy_static;

macro_rules! fatal {
    ($($tt:tt)*) => {{
        use std::io::Write;
        writeln!(&mut ::std::io::stderr(), $($tt)*).unwrap();
        ::std::process::exit(1)
    }}
}


lazy_static! {
    static ref LOG_PATTERN: Regex = Regex::new(r"(?x)
\A
\[(?P<ts>[^\]]+)\]
\s
(?P<log_level>[CIEWTD])
\s
(?:\[MSGID:\s(?P<msg_id>[^\]]+)\]\s)?
\[(?P<file_info>[^\]]+)\]
\s
(?P<domain>[^:]+):
\s
(?P<msg_parts_raw>.+)
").unwrap();
}

fn parse(s: &str) -> Option<HashMap<&str, &str>> {
    LOG_PATTERN.captures(s).map(|cap| {
        let mut hash = HashMap::new();
        for name in LOG_PATTERN.capture_names() {
            if let Some(name) = name {
                if let Some(value) = cap.name(name) {
                    hash.insert(name, value.as_str());
                }
            }
        }
        hash
    })
}

fn process_lines() -> io::Result<()> {
    let stdin = io::stdin();
    for (idx, line_result) in stdin.lock().lines().enumerate() {
        parse(&line_result?).map(|h| println!("{} {:?}", idx + 1, h));
    }
    Ok(())
}

fn main() {
    if let Err(err) = process_lines() {
        fatal!("processing lines: {}", err)
    }

    println!("Done.");
}
