mod utils;

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Error};

use termwiz::escape::parser::Parser;

fn main() -> Result<(), Error> {
    let mut reader = BufReader::new(io::stdin());

    let mut parser = Parser::new();

    let mut f: BufWriter<File>;

    match utils::create_logfile() {
        Ok(file) => f = BufWriter::new(file),
        Err(e) => return Err(e),
    };

    let mut width: usize;
    let mut read_buf = String::new();
    // Read a line from stdin and parse escape sequences in that line
    match reader.read_line(&mut read_buf) {
        Ok(w) => width = w,
        Err(e) => {
            width = 0;
            eprintln!("{}", e);
        }
    }

    while width > 0 {
        let actions = parser.parse_as_vec(&read_buf.as_bytes());
        utils::do_actions(actions, &mut f, width);
        read_buf = String::new();
        match reader.read_line(&mut read_buf) {
            Ok(w) => {
                width = w;
            }
            Err(e) => {
                width = 0;
                eprintln!("{}", e);
            }
        }
    }
    Ok(())
}
