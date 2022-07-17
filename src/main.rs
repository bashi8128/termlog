use std::io::{self, BufRead ,BufReader, Write, BufWriter};
use std::fs;

use chrono::{DateTime, Local};
use termwiz::escape::{Action, ControlCode};
use termwiz::escape::parser::Parser;

fn main() {
    let mut reader = BufReader::new(io::stdin());

    let mut read_buf = String::new();
    let mut parser = Parser::new();

    let mut datetime: DateTime<Local> = Local::now();
    let mut datetime_str: String = datetime.format("%F_%H%M%S%.f")
                                                       .to_string();

    let mut f = BufWriter::new(fs::File::create(datetime_str + ".log")
                                        .unwrap());

    while reader.read_line(&mut read_buf).unwrap() > 0 {
        let mut write_buf = String::new();
        parser.parse(&read_buf.as_bytes(), |action| match action {
            Action::Print(c) => write_buf.push(c),
            Action::Control(ctrl_code) => match ctrl_code {
                ControlCode::Backspace => {write_buf.pop().unwrap();},
                ControlCode::HorizontalTab => write_buf.push('\t'),
                ControlCode::LineFeed => write_buf.push('\n'),
                _ => {}
            },
            _ => {}
        });

        datetime = Local::now();
        datetime_str = datetime.format("[%F %H%M%S%.f] ").to_string();

        write_buf = [datetime_str, write_buf].concat();
        match f.write(write_buf.as_bytes()) {
            Ok(_) => {},
            Err(e) => eprintln!("Error in writing to file: {}", e),
        };
        read_buf = String::new();
    }
}
