//! Universal Linear Framebuffer & 2D Graphics Engine for Zeroth OS
//! Includes flicker-free cursor overlay and dirty rect restoration

use super::font::{get_glyph, FONT_HEIGHT, FONT_WIDTH};

// Claude-inspired Zeroth OS Color Palette (0x00RRGGBB)
pub const COLOR_CREAM: u32 = 0x00F0EEE6;
pub const COLOR_TERRACOTTA: u32 = 0x00D97757;
pub const COLOR_CHARCOAL: u32 = 0x002B2A27;
pub const COLOR_OCHRE: u32 = 0x00D4A373;
pub const COLOR_CARD_BG: u32 = 0x00FAF8F5;
pub const COLOR_MUTED: u32 = 0x007C7871;
pub const COLOR_WHITE: u32 = 0x00FFFFFF;

pub struct CursorOverlay {
    pub last_x: usize,
    pub last_y: usize,
    pub saved_pixels: [u32; 16 * 16],
    pub has_saved: bool,
}

pub static mut CURSOR_OVERLAY: CursorOverlay = CursorOverlay {
    last_x: 0,
    last_y: 0,
    saved_pixels: [0; 256],
    has_saved: false,
};

pub struct Framebuffer {
    pub buffer: *mut u32,
    pub width: usize,
    pub height: usize,
    pub pitch: usize, // pixels per scanline
}

unsafe impl Send for Framebuffer {}
unsafe impl Sync for Framebuffer {}

impl Framebuffer {
    pub const fn empty() -> Self {
        Self {
            buffer: core::ptr::null_mut(),
            width: 0,
            height: 0,
            pitch: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.buffer.is_null() && self.width > 0 && self.height > 0
    }

    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        if x < self.width && y < self.height && !self.buffer.is_null() {
            unsafe { *self.buffer.add(y * self.pitch + x) }
        } else {
            0
        }
    }

    #[inline]
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height && !self.buffer.is_null() {
            unsafe {
                *self.buffer.add(y * self.pitch + x) = color;
            }
        }
    }

    pub fn clear(&mut self, color: u32) {
        if !self.is_valid() {
            return;
        }
        for y in 0..self.height {
            for x in 0..self.width {
                self.draw_pixel(x, y, color);
            }
        }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        for row in y..((y + h).min(self.height)) {
            for col in x..((x + w).min(self.width)) {
                self.draw_pixel(col, row, color);
            }
        }
    }

    pub fn draw_rect_outline(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32, border: usize) {
        self.fill_rect(x, y, w, border, color);
        self.fill_rect(x, y + h.saturating_sub(border), w, border, color);
        self.fill_rect(x, y, border, h, color);
        self.fill_rect(x + w.saturating_sub(border), y, border, h, color);
    }

    pub fn draw_card(&mut self, x: usize, y: usize, w: usize, h: usize, bg: u32, border: u32) {
        self.fill_rect(x, y, w, h, bg);
        self.draw_rect_outline(x, y, w, h, border, 2);
    }

    pub fn draw_circle_outline(&mut self, cx: usize, cy: usize, radius: usize, color: u32) {
        let r2 = (radius * radius) as isize;
        let r_inner2 = ((radius.saturating_sub(2)) * (radius.saturating_sub(2))) as isize;

        let start_x = cx.saturating_sub(radius + 1);
        let end_x = (cx + radius + 1).min(self.width);
        let start_y = cy.saturating_sub(radius + 1);
        let end_y = (cy + radius + 1).min(self.height);

        for py in start_y..end_y {
            for px in start_x..end_x {
                let dx = px as isize - cx as isize;
                let dy = py as isize - cy as isize;
                let d2 = dx * dx + dy * dy;
                if d2 <= r2 && d2 >= r_inner2 {
                    self.draw_pixel(px, py, color);
                }
            }
        }
    }

    pub fn draw_zeroth_logo(&mut self, cx: usize, cy: usize, radius: usize, color: u32, dot_color: u32) {
        let r_inner = radius.saturating_sub(radius / 4);
        let r2_outer = (radius * radius) as isize;
        let r2_inner = (r_inner * r_inner) as isize;

        let start_x = cx.saturating_sub(radius + 2);
        let end_x = (cx + radius + 2).min(self.width);
        let start_y = cy.saturating_sub(radius + 2);
        let end_y = (cy + radius + 2).min(self.height);

        for py in start_y..end_y {
            for px in start_x..end_x {
                let dx = px as isize - cx as isize;
                let dy = py as isize - cy as isize;
                let dist2 = dx * dx + dy * dy;

                let is_gap = dx > (radius as isize / 3) && dy < -(radius as isize / 6);

                if dist2 <= r2_outer && dist2 >= r2_inner && !is_gap {
                    self.draw_pixel(px, py, color);
                }
            }
        }

        let dot_cx = cx + (radius * 7) / 10;
        let dot_cy = cy.saturating_sub((radius * 7) / 10);
        let dot_r = (radius / 5).max(3);
        let dot_r2 = (dot_r * dot_r) as isize;

        for py in dot_cy.saturating_sub(dot_r)..(dot_cy + dot_r).min(self.height) {
            for px in dot_cx.saturating_sub(dot_r)..(dot_cx + dot_r).min(self.width) {
                let dx = px as isize - dot_cx as isize;
                let dy = py as isize - dot_cy as isize;
                if dx * dx + dy * dy <= dot_r2 {
                    self.draw_pixel(px, py, dot_color);
                }
            }
        }
    }

    pub fn draw_char(&mut self, x: usize, y: usize, c: char, color: u32) {
        let glyph = get_glyph(c);
        for row in 0..FONT_HEIGHT {
            let row_byte = glyph[row];
            for col in 0..FONT_WIDTH {
                if (row_byte & (0x80 >> col)) != 0 {
                    self.draw_pixel(x + col, y + row, color);
                }
            }
        }
    }

    pub fn draw_string(&mut self, mut x: usize, y: usize, text: &str, color: u32) {
        for c in text.chars() {
            if c == '\n' {
                break;
            }
            self.draw_char(x, y, c, color);
            x += FONT_WIDTH;
        }
    }

    /// Restores the pixels under the previous cursor position (eliminates full screen redraw flicker!)
    pub fn restore_cursor_overlay(&mut self) {
        unsafe {
            if CURSOR_OVERLAY.has_saved {
                let lx = CURSOR_OVERLAY.last_x;
                let ly = CURSOR_OVERLAY.last_y;
                for row in 0..16 {
                    for col in 0..16 {
                        let c = CURSOR_OVERLAY.saved_pixels[row * 16 + col];
                        self.draw_pixel(lx + col, ly + row, c);
                    }
                }
                CURSOR_OVERLAY.has_saved = false;
            }
        }
    }

    /// Saves the 16x16 pixels at (new_x, new_y) and renders the sharp cursor arrow
    pub fn draw_flicker_free_cursor(&mut self, new_x: usize, new_y: usize) {
        let cursor_bitmap: [u8; 11] = [
            0b10000000,
            0b11000000,
            0b11100000,
            0b11110000,
            0b11111000,
            0b11111100,
            0b11111110,
            0b11110000,
            0b11011000,
            0b10001100,
            0b00001100,
        ];

        unsafe {
            // 1. Save original pixels under cursor
            for row in 0..16 {
                for col in 0..16 {
                    CURSOR_OVERLAY.saved_pixels[row * 16 + col] = self.get_pixel(new_x + col, new_y + row);
                }
            }
            CURSOR_OVERLAY.last_x = new_x;
            CURSOR_OVERLAY.last_y = new_y;
            CURSOR_OVERLAY.has_saved = true;

            // 2. Draw crisp cursor arrow
            for (row, byte) in cursor_bitmap.iter().enumerate() {
                for col in 0..8 {
                    if (byte & (0x80 >> col)) != 0 {
                        // White drop shadow outline
                        self.draw_pixel(new_x + col + 1, new_y + row + 1, COLOR_WHITE);
                        // Deep charcoal arrow
                        self.draw_pixel(new_x + col, new_y + row, COLOR_CHARCOAL);
                    }
                }
            }
        }
    }
}

pub static mut FRAMEBUFFER: Framebuffer = Framebuffer::empty();
