//! 16550 UART Serial Driver for Zeroth OS
//! Enables high-speed kernel logging through COM1 (port 0x3F8)

use core::arch::asm;
use core::fmt;

const COM1: u16 = 0x3F8;

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}

#[inline]
unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self { port }
    }

    /// Initialize UART to 38400 baud, 8 bits, no parity, 1 stop bit
    pub fn init(&self) {
        unsafe {
            outb(self.port + 1, 0x00); // Disable all interrupts
            outb(self.port + 3, 0x80); // Enable DLAB (set baud rate divisor)
            outb(self.port + 0, 0x03); // Divisor 3 -> 38400 baud
            outb(self.port + 1, 0x00);
            outb(self.port + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(self.port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
            outb(self.port + 4, 0x0B); // IRQs enabled, RTS/DSR set
        }
    }

    fn is_transmit_empty(&self) -> bool {
        unsafe { (inb(self.port + 5) & 0x20) != 0 }
    }

    pub fn write_byte(&self, byte: u8) {
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        unsafe {
            outb(self.port, byte);
        }
    }

    pub fn write_string(&self, s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
    }
}

pub static SERIAL: SerialPort = SerialPort::new(COM1);

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! sprint {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut port = $crate::arch::serial::SerialPort::new(0x3F8);
        let _ = write!(port, $($arg)*);
    });
}

#[macro_export]
macro_rules! sprintln {
    () => ($crate::sprint!("\n"));
    ($($arg:tt)*) => ($crate::sprint!("{}\n", format_args!($($arg)*)));
}
