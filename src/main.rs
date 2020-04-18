use std::io;
use std::io::prelude::*;

use regex::Regex;
use serde_json::{Map, Value};

#[macro_use]
extern crate lazy_static;

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
        r#"(?x)\AThe\ message\ "{}"\ repeated\ (?P<repetitions>\S+)\ times\ between\ {}\ and\ {}\z"#,
        *LOG_PATTERN_STRING,
        make_ts_pattern("ts_beg"),
        make_ts_pattern("ts_end"))).unwrap();
}

fn parse(s: &str) -> Value {
    let mut obj = Map::new();

    // Ensurinng "known_format" is the initial key
    obj.insert("known_format".to_string(), Value::Bool(true));

    if let Err(err) = parse_try(s, &mut obj) {
        obj.insert("known_format".to_string(), Value::Bool(false));
        obj.insert("parse_error".to_string(), Value::String(err));
        obj.insert("message".to_string(), Value::String(s.to_string()));
    };
    Value::Object(obj)
}

fn parse_try(s: &str, obj: &mut Map<String, Value>) -> Result<(), String> {
    match match LOG_PATTERN.captures(s) {
        Some(cap) => (&*LOG_PATTERN, Some(cap)),
        None => (&*LOG_REPEAT_PATTERN, LOG_REPEAT_PATTERN.captures(s)),
    } {
        (rx, Some(cap)) => {
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
                                            [key] => {
                                                return Err(format!("fields::{}: missing value",
                                                                   key))
                                            }
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
                                           Value::Number(value.parse::<u32>()
                                               // Let just sloppily be \S+ captured in the regex
                                               // so that here we can do more specific valudation
                                               .map_err(|err|
                                                  format!("repetitions value: {}", err))?.into()
                                               //.expect("only well-formatted numeral should reach here")
                                               ));
                            }
                            _ => {
                                obj.insert(name.to_string(),
                                           Value::String(value.as_str().to_string()));
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        (_, None) => Err("format mismatch".to_string()),
    }
}

fn process_lines() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdin_lk = stdin.lock();
    let mut buf = String::new();

    while stdin_lk.read_line(&mut buf)? > 0 {
        let o = parse(&buf.trim_end());
        println!("{}", o.to_string());
        buf.clear();
    }

    Ok(())
}

fn main() {
    if let Err(err) = process_lines() {
        eprintln!("processing lines: {}", err);
        std::process::exit(1);
    }
}
