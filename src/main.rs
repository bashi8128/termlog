use std::env;
use std::io::{self, BufRead ,BufReader, Error, Write, BufWriter};
use std::fs::{self, File};

use chrono::{DateTime, Local};
use termwiz::escape::{Action, ControlCode};
use termwiz::escape::parser::Parser;

use::termwiz::escape::csi::{CSI, Cursor, Edit, EraseInDisplay, EraseInLine};

fn main() -> Result<(), Error>{
    let mut reader = BufReader::new(io::stdin());

    let mut parser = Parser::new();

    let mut f: BufWriter<File>;

    match create_logfile() {
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
        do_actions(actions, &mut f, width);
        read_buf = String::new();
        match reader.read_line(&mut read_buf) {
            Ok(w) => {
                width = w;
            },
            Err(e) => {
                width = 0;
                eprintln!("{}", e);
            }
        }
    };
    Ok(())
}

/// Create log file under `~/log/YYYY/MM/DD`, with timestamp prefix
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

    let log_name: String = datetime.format(&(dirpath + &"%F_%H%M%S.%f.log"))
                                   .to_string();
    fs::File::create(log_name)
}

fn do_actions(actions: Vec<Action>, file: &mut BufWriter<File>, term_width: usize) {
    let mut buffer = " ".repeat(term_width);
    let mut cursor: usize = 0;
    for action in actions.iter() {
        match action {
            Action::Print(c) => {
                buffer.replace_range(cursor..cursor+1, &c.to_string());
                cursor += 1;
            },
            Action::Control(ctrl_code) => match ctrl_code {
                ControlCode::Backspace => {
                    if cursor > 0 {
                        cursor -= 1;
                    };
                    buffer.replace_range(cursor..cursor+1, " ");
                },
                ControlCode::HorizontalTab => {
                    buffer.replace_range(cursor..cursor+1, "\t");
                    cursor += 1;
                },
                _ => {}
            },
            Action::CSI(csi) => match csi {
                CSI::Cursor(c) => match c {
                    Cursor::Left(n) => {
                        if cursor > 0 {
                            cursor -= *n as usize;
                        }
                    },
                    Cursor::Right(n) => {
                        if cursor < term_width {
                            cursor += *n as usize;
                        }
                    },
                    _ => {}
                },
                CSI::Edit(e) => match e {
                    Edit::EraseInDisplay(ed) => match ed {
                        EraseInDisplay::EraseToEndOfDisplay => {
                            for pos in cursor..term_width+1 {
                                buffer.replace_range(pos..pos, " ");
                            }
                        },
                        EraseInDisplay::EraseToStartOfDisplay => {
                            for pos in 0..cursor+1 {
                                buffer.replace_range(pos..pos, " ");
                            }
                            cursor = 0;
                        },
                        EraseInDisplay::EraseDisplay => {
                            buffer = " ".repeat(term_width);
                            cursor = 0;
                        },
                        EraseInDisplay::EraseScrollback => (),
                    },
                    Edit::EraseInLine(el) => match el {
                        EraseInLine::EraseToEndOfLine => {
                            for pos in cursor..term_width+1 {
                                buffer.replace_range(pos..pos, " ");
                            }
                        },
                        EraseInLine::EraseToStartOfLine => {
                            for pos in 0..cursor+1 {
                                buffer.replace_range(pos..pos, " ");
                            }
                            cursor = 0;
                        },
                        EraseInLine::EraseLine => {
                            buffer = " ".repeat(term_width);
                            cursor = 0;
                        },
                    }
                    _ => {}
                }
                _ => {}
            }
            _ => {}
       };
    }
    let datetime = Local::now();
    let datetime_str = datetime.format("[%F %H:%M:%S.%f] ").to_string();

    buffer = buffer.trim().to_string();
    buffer = [datetime_str, buffer].concat();
    buffer.push('\n');
    match file.write(buffer.as_bytes()) {
        Ok(_) => {},
        Err(e) => eprintln!("Error in writing to file: {}", e),
    };
}
