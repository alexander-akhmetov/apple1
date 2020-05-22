use std::fs;
use std::sync::mpsc::Sender;
use std::thread;

use apple1::{Apple1, Display, Keyboard};

use mos6502::asm::assemble_file;

use clap::{App, Arg};

struct NcursesKeyboard {}

impl NcursesKeyboard {
    fn new() -> NcursesKeyboard {
        NcursesKeyboard {}
    }

    fn start_input_reading(tx: Sender<u8>) {
        loop {
            tx.send(ncurses::getch() as u8).unwrap();
        }
    }
}

impl Keyboard for NcursesKeyboard {
    fn init(&self, tx: Sender<u8>) {
        thread::spawn(move || NcursesKeyboard::start_input_reading(tx));
    }
}

struct NcursesDisplay {}

impl NcursesDisplay {
    fn new() -> NcursesDisplay {
        NcursesDisplay {}
    }
}

impl Display for NcursesDisplay {
    fn init(&self) {
        ncurses::initscr();
        ncurses::resize_term(60, 40);
        ncurses::scrollok(ncurses::stdscr(), true);
        ncurses::noecho();
        ncurses::raw();
    }

    fn stop(&self) {
        ncurses::endwin();
    }

    fn print(&self, c: char) {
        ncurses::addch(c as ncurses::chtype);
        ncurses::refresh();
    }
}

fn main() {
    env_logger::init();

    let matches = App::new("apple1")
        .arg(
            Arg::with_name("address")
                .short("a")
                .help("Load program at address, default: 0x7000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("program")
                .short("p")
                .help("Load additional program to 0x7000, accepts binary or *.asm files")
                .takes_value(true),
        )
        .get_matches();

    let display = Box::new(NcursesDisplay::new());
    let keyboard = Box::new(NcursesKeyboard::new());

    let mut apple1 = Apple1::new(display, keyboard, "sys/wozmon.bin", "sys/replica1.bin");

    if matches.is_present("program") {
        let mut load_program_at = 0x7000;
        if let Some(addr_string) = matches.value_of("address") {
            load_program_at =
                u16::from_str_radix(addr_string, 16).expect("Can't parse HEX start address");
        }

        let original_pc = apple1.cpu.pc;

        let filename = matches.value_of("program").unwrap();
        if filename.ends_with("asm") {
            apple1.load(&assemble_file(filename), load_program_at);
        } else {
            apple1.load(&fs::read(filename).unwrap(), load_program_at);
        }

        apple1.cpu.pc = original_pc;
    }

    apple1.run();
}
