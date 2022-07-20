use std::env;
use std::io::{self, BufRead ,BufReader, Error, Write, BufWriter};
use std::fs::{self, File};

use chrono::{DateTime, Local};
use termwiz::escape::{Action, ControlCode};
use termwiz::escape::parser::Parser;

fn main() -> Result<(), Error>{
    let mut reader = BufReader::new(io::stdin());

    let mut read_buf = String::new();
    let mut parser = Parser::new();

    let mut f: BufWriter<File>;

    match create_logfile() {
        Ok(file) => f = BufWriter::new(file),
        Err(e) => return Err(e),
    };

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

        let datetime = Local::now();
        let datetime_str = datetime.format("[%F %H%M%S%.f] ").to_string();

        write_buf = [datetime_str, write_buf].concat();
        match f.write(write_buf.as_bytes()) {
            Ok(_) => {},
            Err(e) => eprintln!("Error in writing to file: {}", e),
        };
        read_buf = String::new();
    };
    Ok(())
}

fn create_logfile() -> Result<File, Error> {
    let datetime: DateTime<Local> = Local::now();
    let year: String = datetime.format("%Y").to_string();
    let month: String = datetime.format("%m").to_string();
    let date: String = datetime.format("%d").to_string();

    let dirpath: String = env::var("HOME").unwrap() + "/log/" 
                             + &year + "/"
                             + &month + "/"
                             + &date + "/";

    match fs::create_dir_all(&dirpath) {
        Ok(_) => {},
        Err(e) => eprintln!("Error creating directory: {}", e),
    };

    let log_name: String = datetime.format(&(dirpath + &"%F_%H%M%S%.f.log"))
                                   .to_string();
    fs::File::create(log_name)
}
