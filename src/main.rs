use std::fs;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};

extern crate mos6502;
use mos6502::asm::assemble_file;
use mos6502::cpu::CPU;

extern crate ncurses;

extern crate clap;
use clap::{App, Arg};

#[macro_use]
extern crate log;
extern crate env_logger;

// carriage return (ASCII)
const CR: u8 = 0x0D;

// hardware registers (6821 PIA)
// see description below in the Apple1 source
const KBD: u16 = 0xD010;
const KBDCR: u16 = 0xD011;
const DSP: u16 = 0xD012;

// Addresses to put Woz Monitor and BASIC
// programs
const WOZMON_ADDR: u16 = 0xFF00;
const BASIC_ADDR: u16 = 0xE000;

struct Apple1 {
    cpu: CPU,
    disable_screen: bool,
}

impl Apple1 {
    /// Returns an instance of Apple1
    ///
    /// This function creates a new Apple1 emulator instance,
    /// uploads all necessary programs to it (Woz Monitor and BASIC),
    /// and uses ncurses based screen to handle I/O.
    ///
    /// # Arguments
    ///
    /// * `disable_screen` - A boolean, disables ncurses-based screen, useful for debugging
    /// * `wozmon_rom_path` - A string path to a Woz Monitor binary
    /// * `basic_rom_path` - A string path to a Apple-1 BASIC binary
    pub fn new(disable_screen: bool, wozmon_rom_path: &str, basic_rom_path: &str) -> Apple1 {
        let mut apple1 = Apple1 {
            cpu: CPU::new(),
            disable_screen,
        };

        for (file, addr) in &[(basic_rom_path, BASIC_ADDR), (wozmon_rom_path, WOZMON_ADDR)] {
            let rom = fs::read(file).unwrap_or_else(|_| panic!("No such file: {}", file));
            apple1.load(&rom, *addr);
        }

        apple1.set_callbacks();

        apple1
    }

    fn set_callbacks(&mut self) {
        // Callbacks are being used to catch writes and reads
        // to the CPU's memory.
        //
        // Callback returns Option<(addr: u16, value: u8)>.
        // If there is a tuple returned, CPU will write
        // the value to the corresponding memory address.
        //
        // Apple-1 uses two memory addresses (6821 PIA control registers)
        // to handle keyboard input:
        //
        //   * $D010 (KBD): holds a character when KBDCR 7th bit == 1.
        //
        //   * $D011 (KBDCR): 7th bit is set whenever a kiy is pressed
        //                    on a keyboard. It's cleared automatically,
        //                    when CPU reads from KBD
        //
        fn read_callback(addr: u16) -> Option<(u16, u8)> {
            match addr {
                // if CPU reads from KBD, clear KBDCR
                KBD => Some((KBDCR, 0)),
                _ => None,
            }
        };

        self.cpu.memory.register_read_callback(read_callback);

        fn write_callback(addr: u16, value: u8) -> Option<(u16, u8)> {
            match (addr, value) {
                (KBD, _) => Some((KBDCR, 0x80)),
                (DSP, 0) => None,
                (DSP, v) => Some((DSP, v | 0b1000_0000)),
                _ => None,
            }
        };
        self.cpu.memory.register_write_callback(write_callback);
    }

    pub fn run(&mut self) {
        let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();
        if !self.disable_screen {
            ncurses::initscr();
            ncurses::resize_term(60, 40);
            ncurses::scrollok(ncurses::stdscr(), true);
            ncurses::noecho();
            ncurses::raw();
            thread::spawn(move || Apple1::read_input(tx));
        }

        while !self.cpu.step() {
            self.print_output_to_display();

            if let Ok(c) = rx.try_recv() {
                match c {
                    0x03 => break, // ^c
                    0x05 => {
                        self.print_status();
                        continue;
                    } // ^e
                    _ => {}
                };

                self.write_kbd_input(c);
            }
            // todo: remove after proper implementation in the mos6502
            thread::sleep(time::Duration::from_micros(100));
        }
        ncurses::endwin();
    }

    fn char_to_apple1(&self, c: u8) -> u8 {
        // Apple-1 used only uppercase characters,
        // and 7th bit must be set to 1
        let mut c = c.to_ascii_uppercase() as u8;
        if c == 0xA {
            c = CR; // CR instead of NL
        }

        c | 0x80 // apple1 ascii + set bit 7
    }

    fn write_kbd_input(&mut self, c: u8) {
        // Writes user's input to a CPU's memory
        self.cpu.memory.set(KBD, self.char_to_apple1(c));
    }

    fn read_input(tx: Sender<u8>) {
        // Reads user input from keyboard
        loop {
            tx.send(ncurses::getch() as u8).unwrap();
        }
    }

    fn print_status(&mut self) {
        // For debugging, prints current CPU status to a screen and log
        let status = &format!(
            "[apple1] pc=0x{:X} a=0x{:X} x=0x{:X} y=0x{:X} p=0b{:08b} video=0b{:08b} kbd=0b{:08b}",
            self.cpu.pc,
            self.cpu.a,
            self.cpu.x,
            self.cpu.y,
            self.cpu.p,
            self.cpu.memory.get(DSP),
            self.cpu.memory.get(KBD),
        );
        debug!("{}", status);
        if !self.disable_screen {
            ncurses::addstr(status);
            ncurses::addstr("\n");
        };
    }

    /// Loads a binary program to memory
    ///
    /// # Arguments
    ///
    /// * `program`: array of bytes with program
    /// * `addr`: starting address in memory for the program
    pub fn load(&mut self, program: &[u8], addr: u16) {
        self.cpu.load(program, addr);
    }

    fn print_output_to_display(&mut self) {
        // Prints CPU's output to an ncurses screen
        //
        // Apple-1 used two addresses:
        //    * $D012: 7th bit being used to indicate that
        //              the value (bits 6-0) hasn't been
        //              read by the hardware yet
        //    * $D013: set by Woz Monitor
        let value = self.cpu.memory.get(DSP);

        if value & 0b1000_0000 != 0 {
            // there is a character to display
            // clean the DSB bit and print it to the screen
            self.cpu.memory.set(DSP, 0);
            let value = value & 0b0111_1111; // remove 7 bit
            if value == CR {
                // replace Carriage Return symbol with new line
                ncurses::addch('\n' as ncurses::chtype);
            } else {
                ncurses::addch(value as ncurses::chtype);
            }
            info!("[apple1 -> screen] 0x{:X}", value);
            ncurses::refresh();
        } else {
            debug!("no video output");
        }
    }
}

fn main() {
    env_logger::init();

    let matches = App::new("apple1")
        .arg(
            Arg::with_name("disable-screen")
                .short("s")
                .help("Disable ncurses screen"),
        )
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

    let mut apple1 = Apple1::new(
        matches.is_present("disable-screen"),
        "sys/wozmon.bin",
        "sys/replica1.bin",
    );

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
