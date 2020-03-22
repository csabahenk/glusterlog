use std::io;
use std::io::prelude::*;

macro_rules! fatal {
    ($($tt:tt)*) => {{
        use std::io::Write;
        writeln!(&mut ::std::io::stderr(), $($tt)*).unwrap();
        ::std::process::exit(1)
    }}
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
    if let Err(err) = process_lines() {
        fatal!("processing lines: {}", err)
    }

    println!("Done.");
}
