//! ZeroCalc — Desktop Calculator Application

use crate::drivers::framebuffer::*;
use crate::gui::window::{Window, TITLEBAR_HEIGHT};

pub fn render_calc(win: &Window, fb: &mut Framebuffer) {
    if win.is_minimized {
        return;
    }

    win.draw_frame(fb);

    let content_x = win.x + 4;
    let content_y = win.y + TITLEBAR_HEIGHT + 2;
    let content_w = win.width.saturating_sub(8);

    // Calculator Display Screen
    let screen_w = content_w.saturating_sub(24);
    fb.fill_rect(content_x + 12, content_y + 12, screen_w, 42, COLOR_CHARCOAL);
    fb.draw_string(content_x + screen_w.saturating_sub(40), content_y + 24, "3.1415", COLOR_TERRACOTTA);

    // Keypad Grid
    let keys = [
        ["C", "(", ")", "/"],
        ["7", "8", "9", "*"],
        ["4", "5", "6", "-"],
        ["1", "2", "3", "+"],
        ["0", ".", "%", "="],
    ];

    let start_y = content_y + 64;
    let btn_w = (screen_w.saturating_sub(18)) / 4;
    let btn_h = 32;

    for (row_idx, row) in keys.iter().enumerate() {
        for (col_idx, key) in row.iter().enumerate() {
            let bx = content_x + 12 + col_idx * (btn_w + 6);
            let by = start_y + row_idx * (btn_h + 6);

            let is_op = col_idx == 3 || *key == "=";
            let bg = if is_op { COLOR_TERRACOTTA } else { COLOR_WHITE };
            let fg = if is_op { COLOR_WHITE } else { COLOR_CHARCOAL };

            fb.fill_rect(bx, by, btn_w, btn_h, bg);
            fb.draw_rect_outline(bx, by, btn_w, btn_h, COLOR_OCHRE, 1);
            fb.draw_string(bx + btn_w / 2 - 4, by + 8, key, fg);
        }
    }
}
