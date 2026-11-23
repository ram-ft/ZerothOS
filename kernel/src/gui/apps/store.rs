//! ZeroStore — The Official Graphical App Store for Zeroth OS
//! Browse, inspect, and install verified .zero applications

use crate::drivers::framebuffer::*;
use crate::gui::window::{Window, TITLEBAR_HEIGHT};

pub fn render_store(win: &Window, fb: &mut Framebuffer) {
    if win.is_minimized {
        return;
    }

    win.draw_frame(fb);

    let content_x = win.x + 4;
    let content_y = win.y + TITLEBAR_HEIGHT + 2;
    let content_w = win.width.saturating_sub(8);

    // Top banner
    fb.fill_rect(content_x, content_y, content_w, 46, 0x00EFEBE2);
    fb.draw_string(content_x + 16, content_y + 14, "ZeroStore — Discover Native .zero Packages", COLOR_CHARCOAL);

    let apps = [
        ("ZeroCalc.zero", "Precision Calculator & Unit Converter", "Installed", 0x00E4F0D0, 0x002B6010),
        ("ZeroWeb.zero", "Lightweight Web Browser with TLS 1.3", "Installed", 0x00E4F0D0, 0x002B6010),
        ("ZeroPad.zero", "Syntax-Highlighting Code & Note Editor", "GET (Free)", COLOR_TERRACOTTA, COLOR_WHITE),
        ("ZeroPaint.zero", "Hand-Drawn & Vector Drawing Canvas", "GET (Free)", COLOR_TERRACOTTA, COLOR_WHITE),
        ("ZeroMusic.zero", "Minimal Audio Player & DSP Synthesizer", "GET (Free)", COLOR_TERRACOTTA, COLOR_WHITE),
    ];

    let mut item_y = content_y + 56;

    for (name, desc, status, btn_bg, btn_fg) in apps.iter() {
        // App item card
        fb.fill_rect(content_x + 12, item_y, content_w.saturating_sub(24), 48, COLOR_WHITE);
        fb.draw_rect_outline(content_x + 12, item_y, content_w.saturating_sub(24), 48, COLOR_OCHRE, 1);

        // App Icon
        fb.fill_rect(content_x + 22, item_y + 8, 32, 32, 0x00EFEBE2);
        fb.draw_string(content_x + 32, item_y + 16, "0", COLOR_TERRACOTTA);

        // Name & Description
        fb.draw_string(content_x + 64, item_y + 10, name, COLOR_CHARCOAL);
        fb.draw_string(content_x + 64, item_y + 28, desc, COLOR_MUTED);

        // Action Button
        let btn_w = 90;
        let btn_x = content_x + content_w.saturating_sub(btn_w + 32);
        fb.fill_rect(btn_x, item_y + 12, btn_w, 24, *btn_bg);
        fb.draw_string(btn_x + 10, item_y + 17, status, *btn_fg);

        item_y += 58;
    }
}
