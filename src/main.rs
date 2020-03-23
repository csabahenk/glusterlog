use std::io;
use std::io::prelude::*;

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

fn parse(s: &str) -> Option<String> {
    LOG_PATTERN.captures(s).map(|cap| {
        format!("FOUND: ts: {}, log_level: {}, msg_id: {}, file_info: {}, domain: {}, \
                 msg_parts_raw: {}",
                &cap["ts"],
                &cap["log_level"],
                &cap.name("msg_id").map_or("<NONE>", |m| m.as_str()),
                &cap["file_info"],
                &cap["domain"],
                &cap["msg_parts_raw"])
    })
}

fn process_lines() -> io::Result<()> {
    let stdin = io::stdin();
    for (idx, line_result) in stdin.lock().lines().enumerate() {
        parse(&line_result?).map(|s| println!("{} {}", idx + 1, s));
    }
    Ok(())
}

fn main() {
    if let Err(err) = process_lines() {
        fatal!("processing lines: {}", err)
    }

    println!("Done.");
}
