use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};

extern crate mos6502;
use mos6502::cpu::CPU;

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

pub trait Keyboard {
    // Keyboard should send characters as u8 to tx
    fn init(&mut self, tx: Sender<u8>);
    fn write(&self, c: char);
}

pub trait Display {
    fn init(&self);
    fn stop(&self);
    fn print(&self, c: char);
}

pub struct Apple1 {
    pub cpu: CPU,
    pub display: Box<dyn Display>,
    pub keyboard: Box<dyn Keyboard>,
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
    /// * `wozmon_rom_path` - A string path to a Woz Monitor binary
    /// * `basic_rom_path` - A string path to a Apple-1 BASIC binary
    pub fn new(display: Box<dyn Display>, keyboard: Box<dyn Keyboard>) -> Apple1 {
        let mut apple1 = Apple1 {
            cpu: CPU::new(),
            display,
            keyboard,
        };

        apple1.set_callbacks();

        apple1.display.init();

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
        }

        self.cpu.memory.register_read_callback(read_callback);

        fn write_callback(addr: u16, value: u8) -> Option<(u16, u8)> {
            match (addr, value) {
                (KBD, _) => Some((KBDCR, 0x80)),
                (DSP, 0) => None,
                (DSP, v) => Some((DSP, v | 0b1000_0000)),
                _ => None,
            }
        }

        self.cpu.memory.register_write_callback(write_callback);
    }

    pub fn run(&mut self) {
        let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();
        self.keyboard.init(tx);

        loop {
            self.step();
            if let Ok(c) = rx.try_recv() {
                if c == 0x03 {
                    break;
                }
                self.write_kbd_input(c);
            }
            // todo: remove after proper implementation in the mos6502
            thread::sleep(time::Duration::from_micros(100));
        }
    }

    pub fn step(&mut self) {
        self.cpu.step();
        self.print_output_to_display();
    }

    fn char_to_apple1(&self, c: u8) -> u8 {
        // Apple-1 used only uppercase characters,
        // and 7th bit must be set to 1
        let mut c = c.to_ascii_uppercase();
        if c == 0xA {
            c = CR; // CR instead of NL
        }

        c | 0x80 // apple1 ascii + set bit 7
    }

    pub fn write_kbd_input(&mut self, c: u8) {
        // Writes user's input to a CPU's memory
        self.cpu.memory.set(KBD, self.char_to_apple1(c));
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
        // Prints CPU's output to a display
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
                self.display.print('\n');
            } else {
                self.display.print(value as char);
            }
            info!("[apple1 -> screen] 0x{:X}", value);
        } else {
            debug!("no video output");
        }
    }
}
