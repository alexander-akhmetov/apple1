use std::fs;
use std::io::{self, Write};
use std::sync::mpsc::Sender;
use std::thread;

use apple1::{Apple1, Display, Keyboard};

use mos6502::asm::assemble_file;

extern crate clap;

use clap::{Arg, Command};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal;

// Addresses to put Woz Monitor and BASIC
// programs
const WOZMON_ADDR: u16 = 0xFF00;
const BASIC_ADDR: u16 = 0xE000;

struct TermKeyboard {}

impl TermKeyboard {
    fn new() -> TermKeyboard {
        TermKeyboard {}
    }

    fn start_input_reading(tx: Sender<u8>) {
        loop {
            if let Ok(Event::Key(key_event)) = event::read() {
                if key_event.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }
                let byte = match key_event.code {
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        0x03
                    }
                    KeyCode::Char(c) => c as u8,
                    KeyCode::Enter => 0x0A,
                    KeyCode::Backspace => 0x08,
                    KeyCode::Esc => 0x1B,
                    _ => continue,
                };
                if tx.send(byte).is_err() {
                    break;
                }
            }
        }
    }
}

impl Keyboard for TermKeyboard {
    fn init(&mut self, tx: Sender<u8>) {
        thread::spawn(move || TermKeyboard::start_input_reading(tx));
    }

    fn write(&self, _c: char) {}
}

struct TermDisplay {}

impl TermDisplay {
    fn new() -> TermDisplay {
        TermDisplay {}
    }
}

impl Display for TermDisplay {
    fn init(&self) {
        terminal::enable_raw_mode().expect("Failed to enable raw mode");
    }

    fn stop(&self) {
        terminal::disable_raw_mode().expect("Failed to disable raw mode");
    }

    fn print(&self, c: char) {
        let mut stdout = io::stdout();
        if c == '\n' {
            write!(stdout, "\r\n").unwrap();
        } else {
            write!(stdout, "{}", c).unwrap();
        }
        stdout.flush().unwrap();
    }
}

fn main() {
    env_logger::init();

    let matches = Command::new("apple1")
        .arg(
            Arg::new("address")
                .short('a')
                .help("Load program at address, default: 0x7000")
                .num_args(1),
        )
        .arg(
            Arg::new("program")
                .short('p')
                .help("Load additional program to 0x7000, accepts binary or *.asm files")
                .num_args(1),
        )
        .get_matches();

    let display = Box::new(TermDisplay::new());
    let keyboard = Box::new(TermKeyboard::new());

    let mut apple1 = Apple1::new(display, keyboard);

    let replica1_rom = fs::read("sys/replica1.bin").unwrap();
    apple1.load(&replica1_rom, BASIC_ADDR);

    let wozmon_rom = fs::read("sys/wozmon.bin").unwrap();
    apple1.load(&wozmon_rom, WOZMON_ADDR);

    if matches.contains_id("program") {
        let mut load_program_at = 0x7000;
        if let Some(addr_string) = matches.get_one::<String>("address") {
            load_program_at =
                u16::from_str_radix(addr_string, 16).expect("Can't parse HEX start address");
        }

        let original_pc = apple1.cpu.pc;

        let filename = matches.get_one::<String>("program").unwrap();
        if filename.ends_with("asm") {
            apple1.load(&assemble_file(filename), load_program_at);
        } else {
            apple1.load(&fs::read(filename).unwrap(), load_program_at);
        }

        apple1.cpu.pc = original_pc;
    }

    apple1.run();
}
