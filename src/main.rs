use std::io;
use std::io::prelude::*;

use std::error::Error;
use std::io::{Write, stderr};

fn fatal_error(err: &dyn Error, msg: &str) {
    let err_fill = if msg.is_empty() {
        ""
    } else {
        "while "
    };
    let _ = writeln!(stderr(), "error {}{}: {}", err_fill, msg, err);
    std::process::exit(1)
}

fn parse(s: &str) {
    println!("GOT: {}", s);
}

fn process_lines() -> io::Result<()> {
    let stdin = io::stdin();
    for line_result in stdin.lock().lines() {
        parse(&line_result?)
    }
    Ok(())
}

fn main() {
    if let Err(err) =  process_lines() {
        fatal_error(&err, "processing lines")
    }
    println!("Done.");
}
