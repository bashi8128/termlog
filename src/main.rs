use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Error, Write};

use chrono::{DateTime, Local};
use termwiz::escape::parser::Parser;
use termwiz::escape::{Action, ControlCode};

use termwiz::escape::csi::{Cursor, Edit, EraseInDisplay, EraseInLine, CSI};

fn main() -> Result<(), Error> {
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
            }
            Err(e) => {
                width = 0;
                eprintln!("{}", e);
            }
        }
    }
    Ok(())
}

/// Create log file under `~/log/YYYY/MM/DD`, with timestamp prefix
fn create_logfile() -> Result<File, Error> {
    let datetime: DateTime<Local> = Local::now();
    let year: String = datetime.format("%Y").to_string();
    let month: String = datetime.format("%m").to_string();
    let date: String = datetime.format("%d").to_string();

    let dirpath: String =
        env::var("HOME").unwrap() + "/log/" + &year + "/" + &month + "/" + &date + "/";

    match fs::create_dir_all(&dirpath) {
        Ok(_) => {}
        Err(e) => eprintln!("Error creating directory: {}", e),
    };

    let log_name: String = datetime
        .format(&(dirpath + &"%F_%H%M%S.%f.log"))
        .to_string();
    fs::File::create(log_name)
}

/// Execute actions in vector generated by `parse_as_vec`
fn do_actions(actions: Vec<Action>, file: &mut BufWriter<File>, term_width: usize) {
    let mut buffer = " ".repeat(term_width); // Buffer for output filled with white spaces
    let mut cursor: usize = 0; // Cursor position
    for action in actions.iter() {
        match action {
            // Print a single character to the buffer by replacing it with a space
            Action::Print(c) => {
                buffer.replace_range(cursor..cursor + 1, &c.to_string());
                cursor += 1;
            }
            Action::Control(ctrl_code) => match ctrl_code {
                // Move cursor one columns left and replace a character at the cursor with a space
                ControlCode::Backspace => {
                    if cursor > 0 {
                        cursor -= 1;
                    };
                    buffer.replace_range(cursor..cursor + 1, " ");
                }
                // Replace a space with a horizontal tab
                ControlCode::HorizontalTab => {
                    buffer.replace_range(cursor..cursor + 1, "\t");
                    cursor += 1;
                }
                // Move cursor to the start of the line
                ControlCode::CarriageReturn => {
                    cursor = 0;
                }
                _ => {}
            },
            Action::CSI(csi) => match csi {
                CSI::Cursor(c) => match c {
                    // Move cursor left n columns
                    Cursor::Left(n) => {
                        if cursor > 0 {
                            cursor -= *n as usize;
                        }
                    }
                    // Move cursor right n columns
                    Cursor::Right(n) => {
                        if cursor < term_width {
                            cursor += *n as usize;
                        }
                    }
                    _ => {}
                },
                CSI::Edit(e) => match e {
                    Edit::EraseInDisplay(ed) => match ed {
                        // Replace characters from the cursor position
                        // to the end of the line with spaces
                        EraseInDisplay::EraseToEndOfDisplay => {
                            for pos in cursor..term_width + 1 {
                                buffer.replace_range(pos..pos, " ");
                            }
                        }
                        // Replace characters from the start of the line
                        // to the current cursor position with spaces
                        EraseInDisplay::EraseToStartOfDisplay => {
                            for pos in 0..cursor + 1 {
                                buffer.replace_range(pos..pos, " ");
                            }
                            cursor = 0;
                        }
                        // Re-create the write buffer
                        EraseInDisplay::EraseDisplay => {
                            buffer = " ".repeat(term_width);
                            cursor = 0;
                        }
                        EraseInDisplay::EraseScrollback => (),
                    },
                    Edit::EraseInLine(el) => match el {
                        // Replace characters from the cursor position
                        // to the end of the line with spaces
                        EraseInLine::EraseToEndOfLine => {
                            for pos in cursor..term_width + 1 {
                                buffer.replace_range(pos..pos, " ");
                            }
                        }
                        // Replace characters from the start of the line
                        // to the current cursor position with spaces
                        EraseInLine::EraseToStartOfLine => {
                            for pos in 0..cursor + 1 {
                                buffer.replace_range(pos..pos, " ");
                            }
                            cursor = 0;
                        }
                        // Re-create the write buffer
                        EraseInLine::EraseLine => {
                            buffer = " ".repeat(term_width);
                            cursor = 0;
                        }
                    },
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        };
    }
    let datetime = Local::now();
    let datetime_str = datetime.format("[%F %H:%M:%S.%f] ").to_string();

    buffer = buffer.trim().to_string();
    buffer = [datetime_str, buffer].concat();
    buffer.push('\n');
    match file.write(buffer.as_bytes()) {
        Ok(_) => {}
        Err(e) => eprintln!("Error in writing to file: {}", e),
    };
}
