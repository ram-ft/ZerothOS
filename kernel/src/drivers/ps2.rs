//! PS/2 Keyboard and Mouse Driver for Zeroth OS
//! Fully supports Shift keys, symbols, capital letters, Backspace, and Enter

use core::arch::asm;

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

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

pub struct MouseState {
    pub x: isize,
    pub y: isize,
    pub left_button: bool,
    pub right_button: bool,
    pub left_clicked: bool,
}

pub static mut MOUSE: MouseState = MouseState {
    x: 400,
    y: 300,
    left_button: false,
    right_button: false,
    left_clicked: false,
};

static mut MOUSE_CYCLE: u8 = 0;
static mut MOUSE_BYTES: [u8; 3] = [0; 3];
static mut SHIFT_PRESSED: bool = false;

pub fn init_ps2(screen_width: usize, screen_height: usize) {
    unsafe {
        MOUSE.x = (screen_width / 2) as isize;
        MOUSE.y = (screen_height / 2) as isize;
        MOUSE.left_button = false;
        MOUSE.left_clicked = false;

        wait_write();
        outb(PS2_STATUS_PORT, 0xA8); // Enable secondary mouse

        wait_write();
        outb(PS2_STATUS_PORT, 0x20); // Read controller configuration
        wait_read();
        let status = inb(PS2_DATA_PORT) | 2;

        wait_write();
        outb(PS2_STATUS_PORT, 0x60); // Write controller configuration
        wait_write();
        outb(PS2_DATA_PORT, status);

        mouse_write(0xF6); // Set defaults
        let _ = mouse_read();

        mouse_write(0xF4); // Enable data reporting
        let _ = mouse_read();
    }
}

unsafe fn wait_write() {
    let mut timeout = 100_000;
    while (inb(PS2_STATUS_PORT) & 2) != 0 && timeout > 0 {
        timeout -= 1;
    }
}

unsafe fn wait_read() {
    let mut timeout = 100_000;
    while (inb(PS2_STATUS_PORT) & 1) == 0 && timeout > 0 {
        timeout -= 1;
    }
}

unsafe fn mouse_write(write: u8) {
    wait_write();
    outb(PS2_STATUS_PORT, 0xD4);
    wait_write();
    outb(PS2_DATA_PORT, write);
}

unsafe fn mouse_read() -> u8 {
    wait_read();
    inb(PS2_DATA_PORT)
}

/// Polls for keyboard and mouse events
pub fn poll_ps2_events(max_w: usize, max_h: usize) -> (Option<char>, bool, bool) {
    let mut typed_char = None;
    let mut mouse_moved = false;
    let mut clicked = false;

    unsafe {
        while (inb(PS2_STATUS_PORT) & 1) != 0 {
            let status = inb(PS2_STATUS_PORT);
            let data = inb(PS2_DATA_PORT);

            if (status & 0x20) != 0 {
                // Mouse packet byte
                match MOUSE_CYCLE {
                    0 => {
                        if (data & 0x08) != 0 {
                            MOUSE_BYTES[0] = data;
                            MOUSE_CYCLE = 1;
                        }
                    }
                    1 => {
                        MOUSE_BYTES[1] = data;
                        MOUSE_CYCLE = 2;
                    }
                    2 => {
                        MOUSE_BYTES[2] = data;
                        MOUSE_CYCLE = 0;

                        let flags = MOUSE_BYTES[0];
                        let mut dx = MOUSE_BYTES[1] as isize;
                        let mut dy = MOUSE_BYTES[2] as isize;

                        if (flags & 0x10) != 0 { dx |= !0xFF; }
                        if (flags & 0x20) != 0 { dy |= !0xFF; }

                        let prev_x = MOUSE.x;
                        let prev_y = MOUSE.y;

                        // Clamp to screen
                        MOUSE.x = (MOUSE.x + dx).clamp(0, (max_w.saturating_sub(16)) as isize);
                        MOUSE.y = (MOUSE.y - dy).clamp(0, (max_h.saturating_sub(16)) as isize);

                        if MOUSE.x != prev_x || MOUSE.y != prev_y {
                            mouse_moved = true;
                        }

                        let prev_btn = MOUSE.left_button;
                        MOUSE.left_button = (flags & 1) != 0;
                        MOUSE.right_button = (flags & 2) != 0;

                        // Click event on press down
                        if MOUSE.left_button && !prev_btn {
                            clicked = true;
                            MOUSE.left_clicked = true;
                        }
                    }
                    _ => MOUSE_CYCLE = 0,
                }
            } else {
                // Keyboard scancode (Set 1)
                if data == 0x2A || data == 0x36 {
                    // Left / Right Shift pressed
                    SHIFT_PRESSED = true;
                } else if data == 0xAA || data == 0xB6 {
                    // Left / Right Shift released
                    SHIFT_PRESSED = false;
                } else if (data & 0x80) == 0 {
                    // Key press
                    if let Some(c) = scancode_to_char(data, SHIFT_PRESSED) {
                        typed_char = Some(c);
                    }
                }
            }
        }
    }

    (typed_char, mouse_moved, clicked)
}

fn scancode_to_char(scancode: u8, shift: bool) -> Option<char> {
    if !shift {
        match scancode {
            0x1E => Some('a'), 0x30 => Some('b'), 0x2E => Some('c'), 0x20 => Some('d'),
            0x12 => Some('e'), 0x21 => Some('f'), 0x22 => Some('g'), 0x23 => Some('h'),
            0x17 => Some('i'), 0x24 => Some('j'), 0x25 => Some('k'), 0x26 => Some('l'),
            0x32 => Some('m'), 0x31 => Some('n'), 0x18 => Some('o'), 0x19 => Some('p'),
            0x10 => Some('q'), 0x13 => Some('r'), 0x1F => Some('s'), 0x14 => Some('t'),
            0x16 => Some('u'), 0x2F => Some('v'), 0x11 => Some('w'), 0x2D => Some('x'),
            0x15 => Some('y'), 0x2C => Some('z'),
            0x02 => Some('1'), 0x03 => Some('2'), 0x04 => Some('3'), 0x05 => Some('4'),
            0x06 => Some('5'), 0x07 => Some('6'), 0x08 => Some('7'), 0x09 => Some('8'),
            0x0A => Some('9'), 0x0B => Some('0'),
            0x39 => Some(' '), 0x1C => Some('\n'), 0x0E => Some('\x08'),
            0x33 => Some(','), 0x34 => Some('.'), 0x35 => Some('/'), 0x0C => Some('-'),
            0x0D => Some('='), 0x1A => Some('['), 0x1B => Some(']'), 0x27 => Some(';'),
            0x28 => Some('\''), 0x29 => Some('`'), 0x2B => Some('\\'),
            _ => None,
        }
    } else {
        match scancode {
            0x1E => Some('A'), 0x30 => Some('B'), 0x2E => Some('C'), 0x20 => Some('D'),
            0x12 => Some('E'), 0x21 => Some('F'), 0x22 => Some('G'), 0x23 => Some('H'),
            0x17 => Some('I'), 0x24 => Some('J'), 0x25 => Some('K'), 0x26 => Some('L'),
            0x32 => Some('M'), 0x31 => Some('N'), 0x18 => Some('O'), 0x19 => Some('P'),
            0x10 => Some('Q'), 0x13 => Some('R'), 0x1F => Some('S'), 0x14 => Some('T'),
            0x16 => Some('U'), 0x2F => Some('V'), 0x11 => Some('W'), 0x2D => Some('X'),
            0x15 => Some('Y'), 0x2C => Some('Z'),
            0x02 => Some('!'), 0x03 => Some('@'), 0x04 => Some('#'), 0x05 => Some('$'),
            0x06 => Some('%'), 0x07 => Some('^'), 0x08 => Some('&'), 0x09 => Some('*'),
            0x0A => Some('('), 0x0B => Some(')'),
            0x39 => Some(' '), 0x1C => Some('\n'), 0x0E => Some('\x08'),
            0x33 => Some('<'), 0x34 => Some('>'), 0x35 => Some('?'), 0x0C => Some('_'),
            0x0D => Some('+'), 0x1A => Some('{'), 0x1B => Some('}'), 0x27 => Some(':'),
            0x28 => Some('"'), 0x29 => Some('~'), 0x2B => Some('|'),
            _ => None,
        }
    }
}
