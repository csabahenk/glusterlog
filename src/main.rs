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

fn make_ts_pattern(s: &str) -> String {
    format!(r"\[(?P<{}>[^\]]+)\]", s)
}

lazy_static! {
    static ref LOG_PATTERN_STRING: &'static str = r"(?P<log_level>[CIEWTD])
\s
(?:\[MSGID:\s(?P<msg_id>[^\]]+)\]\s)?
\[(?P<file_info>[^\]]+)\]
\s
(?P<domain>[^:]+):
\s
(?P<msg_parts_raw>.+)";

    static ref LOG_PATTERN: Regex = Regex::new(&format!(r"(?x)\A{}\s{}",
        make_ts_pattern("ts"),
        *LOG_PATTERN_STRING)).unwrap();

    static ref LOG_REPEAT_PATTERN: Regex = Regex::new(&format!(
        r#"(?x)\AThe\ message\ "{}"\ repeated\ (?P<repetitions>\d+)\ times\ between\ {}\ and\ {}\z"#,
        *LOG_PATTERN_STRING,
        make_ts_pattern("ts_beg"),
        make_ts_pattern("ts_end"))).unwrap();
}

fn parse(s: &str) -> Value {
    let mut obj = Map::new();

    match match LOG_PATTERN.captures(s) {
        Some(cap) => (&*LOG_PATTERN, Some(cap)),
        None => (&*LOG_REPEAT_PATTERN, LOG_REPEAT_PATTERN.captures(s)),
    } {
        (rx, Some(cap)) => {
            obj.insert("known_format".to_string(), Value::Bool(true));
            for name in rx.capture_names() {
                if let Some(name) = name {
                    if let Some(value) = cap.name(name) {
                        match name {
                            // "msg_parts_raw needs" postprocessing
                            "msg_parts_raw" => {
                                // ... split it with tabs
                                let mut splitter = value.as_str().split('\t');
                                if let Some(msg) = splitter.next() {
                                    // first entry is value for "meesage" key
                                    obj.insert("message".to_string(),
                                               Value::String(msg.to_string()));
                                    // consecutive entries are 'key=val' pairs, which
                                    // are collected under "fields" key
                                    let mut fields = Map::new();
                                    for msg_part in splitter {
                                        match msg_part.splitn(2, '=').collect::<Vec<&str>>()[..] {
                                            [key, value] => {
                                                fields.insert(key.to_string(),
                                                              Value::String(value.to_string()))
                                            }
                                            [key] => fields.insert(key.to_string(), Value::Null),
                                            _ => panic!("splitn(2,..) result arity not 1 or 2"),
                                        };
                                    }
                                    if !fields.is_empty() {
                                        obj.insert("fields".to_string(), Value::Object(fields));
                                    }
                                }
                            }
                            "repetitions" => {
                                let value = value.as_str();
                                obj.insert(name.to_string(),
                                           match value.parse() {
                                               Ok(n) => Value::Number(n),
                                               _ => Value::String(value.to_string()),
                                           });
                            }
                            _ => {
                                obj.insert(name.to_string(),
                                           Value::String(value.as_str().to_string()));
                            }
                        }
                    }
                }
            }
        }
        (_, None) => {
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
