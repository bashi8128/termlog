//! Author: Masahiro Itabashi <itabasi.lm@gmail.com>
//! Last modified: Sun, 29 Sep 2019 22:07:00 +0900
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Error, Write};

use chrono::{DateTime, Local};

use termwiz::escape::csi::{Cursor, Edit, EraseInDisplay, EraseInLine, CSI};
use termwiz::escape::{Action, ControlCode};

/// Execute actions in vector generated by `parse_as_vec`
pub fn do_actions(actions: Vec<Action>, file: &mut BufWriter<File>, term_width: usize) {
    let mut buffer = " ".repeat(term_width*10).to_string(); // Buffer for output filled with white spaces
    let mut cursor: usize = 0; // Cursor position
    for action in actions.iter() {
        match action {
            // Print a single character to the buffer by replacing it with a space
            Action::Print(c) => {
                // FIXME: Issue#5: Panic may occur here once 'c' is a multibyte character
                buffer.replace_range(cursor..cursor + 1, &c.to_string());
                cursor += 1;
            }
            Action::Control(ctrl_code) => {
                cursor = reflect_control_codes(ctrl_code, &mut buffer, cursor);
            }
            Action::CSI(csi) => {
                cursor = reflect_csi_seqs(csi, &mut buffer, cursor);
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
        Ok(_) => {}
        Err(e) => eprintln!("Error in writing to file: {}", e),
    };
}

/// Create log file under `~/log/YYYY/MM/DD`, with timestamp prefix
pub fn create_logfile() -> Result<File, Error> {
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

/// Reflects a given C0 and C1 control code to the buffer and returns cursor position.
/// This function ignore below control codes.
///
/// # C0 Control Codes
/// - 0x00: NUL(Null)  
/// - 0x01: SOH(Start of Heading)  
/// - 0x02: STX(Start of Text)  
/// - 0x03: ETX(End of Text)  
/// - 0x04: EOT(Endo of Transmission)  
/// - 0x05: ENQ(Enquire)  
/// - 0x06: ACK(Acknowledge)  
/// - 0x07: BEL(Bell)  
/// - 0x0e: SO(Shift Out)  
/// - 0x0f: SI(Shift In)  
/// - 0x10: DLE(Data Link Escape)  
/// - 0x11: Device Control One(DC1)  
/// - 0x12: Device Control Two(DC2)  
/// - 0x13: Device Control Three(DC3)  
/// - 0x14: Device Control Four(DC4)  
/// - 0x15: NAK(Negative Acknowledge)  
/// - 0x16: SYN(Synchronous Idle)  
/// - 0x17: ETB(End of Transmission Block)  
/// - 0x18: CAN(Cancel)  
/// - 0x19: EM(End of Medium)  
/// - 0x1a: SUB(Substitute)  
/// - 0x1b: ESC(Escape)  
/// - 0x1c: FS(File Separator)  
/// - 0x1d: GS(Group Separator)  
/// - 0x1e: RS(Record Separator)  
/// - 0x1f: US(Unit Separator)  
/// # C1 Control Codes
/// - 0x80: PAD(Padding Character)  
/// - 0x81: HOP(High Octet Preset)  
/// - 0x82: BPH(Break Permitted Here)  
/// - 0x83: NBH(No Break Here)  
/// - 0x84: IND(Index)  
/// - 0x85: NEL(Next Line)  
/// - 0x86: SSA(Start of Selected Area)  
/// - 0x87: ESA(End of Selected Area)  
/// - 0x88: HTS(Horizontal Tabulation Set)  
/// - 0x89: HTJ(Horizontal Tabulation With Justification)  
/// - 0x8a: VTS(Vertical Tabulation Set)  
/// - 0x8b: PLD(Partial Line Down)  
/// - 0x8c: PLU(Partial Line Up)  
/// - 0x8d: RI(Reverse Index)  
/// - 0x8e: SS2(Single-Shift 2)  
/// - 0x8f: SS3(Single-Shift 3)  
/// - 0x90: DCS(Device Control String)  
/// - 0x91: PU1(Private Use 1)  
/// - 0x92: PU2(Private Use 2)  
/// - 0x93: STS(Set Transmnit State)  
/// - 0x94: CCH(Cancel Character)  
/// - 0x95: MW(Message Waiting)  
/// - 0x96: SPA(Start of Protected Area)  
/// - 0x97: EPA(End of Protected Area)  
/// - 0x98: SOS(Start of String)  
/// - 0x99: SGCI(Single Grahpic Character Introducer)
/// - 0x9a: SCI(Single Character Introducer)  
/// - 0x9b: CSI(Control Sequence Introducer)  
/// - 0x9c: ST(String Terminator)  
/// - 0x9d: OSC(Operating System Command)  
/// - 0x9e: PM(Privacy Message)  
/// - 0x9f: APC(Application Program Command)  
pub fn reflect_control_codes(ctrl_code: &ControlCode, buffer: &mut String, cursor: usize) -> usize {
    let mut new_cursor = cursor;
    match ctrl_code {
        // Move cursor one columns left and replace a character at the cursor with a space
        ControlCode::Backspace => {
            if new_cursor > 0 {
                new_cursor -= 1;
            };
            buffer.replace_range(new_cursor..new_cursor + 1, " ");
        }
        // Replace a space with a horizontal tab
        ControlCode::HorizontalTab => {
            buffer.replace_range(new_cursor..new_cursor + 1, "\t");
            new_cursor += 1;
        }
        ControlCode::CarriageReturn => {
            new_cursor = 0;
        }
        _ => {}
    }
    new_cursor
}

/// Reflects a given CSI sequence to the buffer and returns cursor position.
pub fn reflect_csi_seqs(csi: &CSI, buffer: &mut String, cursor: usize) -> usize {
    let mut new_cursor = cursor;
    let term_width = buffer.len();
    match csi {
        CSI::Cursor(c) => match c {
            // Move cursor left n columns
            Cursor::Left(n) => {
                if new_cursor > *n as usize {
                    new_cursor -= *n as usize;
                }
                else {
                    new_cursor = 0;
                }
            }
            // Move cursor right n columns
            Cursor::Right(n) => {
                if new_cursor < term_width {
                    new_cursor += *n as usize;
                }
            }
            _ => {}
        },
        CSI::Edit(e) => match e {
            Edit::EraseInDisplay(ed) => match ed {
                // Replace characters from the cursor position
                // to the end of the line with spaces
                EraseInDisplay::EraseToEndOfDisplay => {
                    let range = new_cursor..term_width;
                    let length = range.len();
                    buffer.replace_range(range, &" ".repeat(length));
                }
                // Replace characters from the start of the line
                // to the current cursor position with spaces
                EraseInDisplay::EraseToStartOfDisplay => {
                    let range = 0..term_width;
                    let length = range.len();
                    buffer.replace_range(range, &" ".repeat(length));
                    new_cursor = 0;
                }
                // Re-create the write buffer
                EraseInDisplay::EraseDisplay => {
                    *buffer = " ".repeat(term_width).to_string();
                    new_cursor = 0;
                }
                EraseInDisplay::EraseScrollback => (),
            },
            Edit::EraseInLine(el) => match el {
                // Replace characters from the cursor position
                // to the end of the line with spaces
                EraseInLine::EraseToEndOfLine => {
                    let range = (new_cursor+1)..term_width;
                    let length = range.len();
                    buffer.replace_range(range, &" ".repeat(length));
                }
                // Replace characters from the start of the line
                // to the current cursor position with spaces
                EraseInLine::EraseToStartOfLine => {
                    let range = 0..term_width;
                    let length = range.len();
                    buffer.replace_range(range, &" ".repeat(length));
                    new_cursor = 0;
                }
                // Re-create the write buffer
                EraseInLine::EraseLine => {
                    *buffer = " ".repeat(term_width).to_string();
                    new_cursor = 0;
                }
            },
            _ => {}
        },
        _ => {}
    }
    new_cursor
}
