//! ZeroGUI Desktop Shell — Top Bar, Wallpaper, Bottom Dock, and Mouse Cursor

use crate::drivers::framebuffer::*;
use crate::drivers::font::FONT_WIDTH;

pub const TOPBAR_HEIGHT: usize = 28;
pub const DOCK_HEIGHT: usize = 46;

pub fn draw_desktop_background(fb: &mut Framebuffer) {
    // 1. Warm Cream Desktop Base
    fb.clear(COLOR_CREAM);

    let w = fb.width;
    let h = fb.height;

    // 2. Artistic Claude-style subtle celestial orbits in background
    let cx = w / 2;
    let cy = (h / 2).saturating_sub(20);

    fb.draw_circle_outline(cx, cy, 180, 0x00E2DDD2);
    fb.draw_circle_outline(cx, cy, 280, 0x00E7E2D8);

    draw_sparkle(fb, cx.saturating_sub(320), cy.saturating_sub(120), COLOR_OCHRE);
    draw_sparkle(fb, cx + 340, cy.saturating_sub(160), COLOR_TERRACOTTA);
    draw_sparkle(fb, cx.saturating_sub(220), cy + 180, COLOR_MUTED);
    draw_sparkle(fb, cx + 260, cy + 140, COLOR_OCHRE);

    // 3. Central Zeroth Desktop Watermark
    fb.draw_zeroth_logo(cx, cy.saturating_sub(40), 44, COLOR_TERRACOTTA, COLOR_CHARCOAL);
    let title = "ZEROTH OS";
    let tx = cx.saturating_sub((title.len() * FONT_WIDTH) / 2);
    fb.draw_string(tx, cy + 20, title, COLOR_CHARCOAL);

    let sub = "Independent Scratch OS . x86_64";
    let sx = cx.saturating_sub((sub.len() * FONT_WIDTH) / 2);
    fb.draw_string(sx, cy + 42, sub, COLOR_TERRACOTTA);
}

fn draw_sparkle(fb: &mut Framebuffer, x: usize, y: usize, color: u32) {
    fb.draw_pixel(x, y, color);
    fb.draw_pixel(x + 1, y, color);
    fb.draw_pixel(x.saturating_sub(1), y, color);
    fb.draw_pixel(x, y + 1, color);
    fb.draw_pixel(x, y.saturating_sub(1), color);
}

pub fn draw_top_bar(fb: &mut Framebuffer, active_title: &str) {
    let w = fb.width;

    fb.fill_rect(0, 0, w, TOPBAR_HEIGHT, COLOR_CHARCOAL);
    fb.fill_rect(0, TOPBAR_HEIGHT.saturating_sub(1), w, 1, COLOR_TERRACOTTA);

    fb.fill_rect(10, 6, 16, 16, COLOR_TERRACOTTA);
    fb.draw_string(14, 6, "0", COLOR_WHITE);
    fb.draw_string(34, 7, "Zeroth OS", COLOR_WHITE);
    fb.draw_string(118, 7, "|", COLOR_MUTED);
    fb.draw_string(130, 7, active_title, COLOR_OCHRE);

    let time_str = "22:20";
    let tray_right = w.saturating_sub(16);
    let time_x = tray_right.saturating_sub(time_str.len() * FONT_WIDTH);
    fb.draw_string(time_x, 7, time_str, COLOR_WHITE);

    let battery_str = "[ 85% ]";
    let batt_x = time_x.saturating_sub((battery_str.len() * FONT_WIDTH) + 12);
    fb.draw_string(batt_x, 7, battery_str, 0x00A0D080);

    let wifi_str = "WiFi: OK";
    let wifi_x = batt_x.saturating_sub((wifi_str.len() * FONT_WIDTH) + 12);
    fb.draw_string(wifi_x, 7, wifi_str, COLOR_OCHRE);
}

pub fn draw_bottom_dock(fb: &mut Framebuffer, active_app: usize) {
    let w = fb.width;
    let h = fb.height;

    let dock_w = 620.min(w.saturating_sub(20));
    let dock_x = (w.saturating_sub(dock_w)) / 2;
    let dock_y = h.saturating_sub(DOCK_HEIGHT + 10);

    fb.fill_rect(dock_x + 4, dock_y + 4, dock_w, DOCK_HEIGHT, 0x00D6D1C5);
    fb.fill_rect(dock_x, dock_y, dock_w, DOCK_HEIGHT, COLOR_CARD_BG);
    fb.draw_rect_outline(dock_x, dock_y, dock_w, DOCK_HEIGHT, COLOR_TERRACOTTA, 2);

    let items = [
        ("Web Browser", "W", 0),
        ("Terminal", ">", 1),
        ("App Store", "S", 2),
        ("Calculator", "+", 3),
        ("Files", "F", 4),
    ];

    let item_slot_w = dock_w / items.len();

    for (i, (label, icon, app_id)) in items.iter().enumerate() {
        let ix = dock_x + i * item_slot_w + 8;
        let iy = dock_y + 6;
        let is_active = *app_id == active_app;

        let bg = if is_active { COLOR_TERRACOTTA } else { 0x00EFEBE2 };
        let fg = if is_active { COLOR_WHITE } else { COLOR_CHARCOAL };

        fb.fill_rect(ix, iy, 34, 34, bg);
        fb.draw_rect_outline(ix, iy, 34, 34, if is_active { COLOR_CHARCOAL } else { COLOR_OCHRE }, 1);
        fb.draw_string(ix + 12, iy + 9, icon, fg);

        let lx = ix + 40;
        let ly = iy + 10;
        fb.draw_string(lx, ly, label, fg);

        if is_active {
            fb.fill_rect(ix + 15, dock_y + DOCK_HEIGHT - 4, 4, 3, COLOR_CHARCOAL);
        }
    }
}

pub fn get_dock_clicked_app(mx: usize, my: usize, screen_w: usize, screen_h: usize) -> Option<usize> {
    let dock_w = 620.min(screen_w.saturating_sub(20));
    let dock_x = (screen_w.saturating_sub(dock_w)) / 2;
    let dock_y = screen_h.saturating_sub(DOCK_HEIGHT + 10);

    if my >= dock_y && my <= dock_y + DOCK_HEIGHT && mx >= dock_x && mx <= dock_x + dock_w {
        let rel_x = mx - dock_x;
        let item_slot_w = dock_w / 5;
        let index = rel_x / item_slot_w;
        if index < 5 {
            return Some(index);
        }
    }
    None
}
