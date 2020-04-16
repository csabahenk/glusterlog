use std::io;
use std::io::prelude::*;

use regex::Regex;
use serde_json::{Map, Value};

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

fn parse(s: &str) -> Value {
    let mut obj = Map::new();

    match LOG_PATTERN.captures(s) {
        Some(cap) => {
            obj.insert("known_format".to_string(), Value::Bool(true));
            for name in LOG_PATTERN.capture_names() {
                if let Some(name) = name {
                    if let Some(value) = cap.name(name) {
                        obj.insert(name.to_string(), Value::String(value.as_str().to_string()));
                    }
                }
            }
        }
        None => {
            obj.insert("known_format".to_string(), Value::Bool(false));
            obj.insert("message".to_string(), Value::String(s.to_string()));
        }
    };
    Value::Object(obj)
}

fn process_lines() -> io::Result<()> {
    let stdin = io::stdin();
    for line_result in stdin.lock().lines() {
        let o = parse(&line_result?);
        println!("{}", o.to_string());
    }
    Ok(())
}

fn main() {
    if let Err(err) = process_lines() {
        fatal!("processing lines: {}", err)
    }
}
