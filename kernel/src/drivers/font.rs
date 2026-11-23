//! Built-in 8x16 Bitmap Font Engine for Zeroth OS
//! Renders readable ASCII glyphs directly onto the linear framebuffer

pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;

// Basic ASCII 8x16 glyph bitmap subset for system text and UI
pub fn get_glyph(c: char) -> &'static [u8; 16] {
    let ascii = (c as usize) & 0x7F;
    if ascii < 32 || ascii > 126 {
        return &GLYPH_UNKNOWN;
    }
    &FONT_DATA[ascii - 32]
}

static GLYPH_UNKNOWN: [u8; 16] = [
    0x7E, 0x81, 0xA5, 0x81, 0xBD, 0xBD, 0x81, 0xA5,
    0x81, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// Essential printable ASCII characters (Space through ~)
// Each byte represents one horizontal row of 8 pixels.
include!("font_data.rs");
