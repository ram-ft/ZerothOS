//! ZeroWeb — Interactive Multi-Page Web Browser for Zeroth OS
//! Clickable navigation buttons, tabs, interactive pages (Docs, Store, Releases, Search)

use crate::drivers::framebuffer::*;
use crate::drivers::font::FONT_WIDTH;
use crate::gui::window::{Window, TITLEBAR_HEIGHT};

#[derive(Copy, Clone, PartialEq)]
pub enum BrowserPage {
    Portal,
    Docs,
    StoreCatalog,
    Releases,
    Search,
}

pub struct BrowserState {
    pub active_page: BrowserPage,
    pub url_text: &'static str,
    pub status_text: &'static str,
}

pub static mut BROWSER: BrowserState = BrowserState {
    active_page: BrowserPage::Portal,
    url_text: "https://zeroth.os/portal",
    status_text: "Connected: TLS 1.3 Active | Status: 200 OK",
};

pub fn render_browser(win: &Window, fb: &mut Framebuffer) {
    if win.is_minimized {
        return;
    }

    win.draw_frame(fb);

    let content_x = win.x + 4;
    let mut cur_y = win.y + TITLEBAR_HEIGHT + 2;
    let content_w = win.width.saturating_sub(8);

    unsafe {
        let browser = &BROWSER;

        // 1. Browser Tab Bar
        fb.fill_rect(content_x, cur_y, content_w, 24, 0x00E2DDD2);

        // Tab 1: Portal
        let tab1_bg = if browser.active_page == BrowserPage::Portal { COLOR_CARD_BG } else { 0x00D6D0C3 };
        fb.fill_rect(content_x + 6, cur_y + 2, 140, 22, tab1_bg);
        fb.draw_string(content_x + 14, cur_y + 6, "Zeroth Portal", COLOR_CHARCOAL);

        // Tab 2: Active sub-page tab
        let tab2_title = match browser.active_page {
            BrowserPage::Docs => "Documentation",
            BrowserPage::StoreCatalog => "ZeroStore Web",
            BrowserPage::Releases => "Release Notes",
            BrowserPage::Search => "Search Results",
            _ => "New Tab",
        };
        let tab2_bg = if browser.active_page != BrowserPage::Portal { COLOR_CARD_BG } else { 0x00D6D0C3 };
        fb.fill_rect(content_x + 150, cur_y + 2, 140, 22, tab2_bg);
        fb.draw_string(content_x + 158, cur_y + 6, tab2_title, COLOR_CHARCOAL);

        cur_y += 24;

        // 2. Navigation Toolbar
        fb.fill_rect(content_x, cur_y, content_w, 36, 0x00ECE7DC);
        fb.fill_rect(content_x, cur_y + 35, content_w, 1, 0x00D0CABE);

        let btn_y = cur_y + 6;
        draw_nav_btn(fb, content_x + 10, btn_y, "<");
        draw_nav_btn(fb, content_x + 36, btn_y, ">");
        draw_nav_btn(fb, content_x + 62, btn_y, "R");
        draw_nav_btn(fb, content_x + 88, btn_y, "H");

        // Address Bar
        let url_x = content_x + 118;
        let url_w = content_w.saturating_sub(180);
        fb.fill_rect(url_x, cur_y + 6, url_w, 24, COLOR_WHITE);
        fb.draw_rect_outline(url_x, cur_y + 6, url_w, 24, COLOR_OCHRE, 1);
        fb.draw_string(url_x + 10, cur_y + 10, browser.url_text, COLOR_CHARCOAL);

        // Security Lock Pill
        let lock_x = url_x + url_w.saturating_sub(64);
        fb.fill_rect(lock_x, cur_y + 8, 56, 20, 0x00E4F0D0);
        fb.draw_string(lock_x + 6, cur_y + 10, "Secure", 0x002B6010);

        // Go button
        let go_x = url_x + url_w + 10;
        fb.fill_rect(go_x, cur_y + 6, 42, 24, COLOR_TERRACOTTA);
        fb.draw_string(go_x + 14, cur_y + 10, "Go", COLOR_WHITE);

        cur_y += 36;

        // 3. Web Page Content Area
        let page_h = (win.y + win.height).saturating_sub(cur_y + 22);
        fb.fill_rect(content_x, cur_y, content_w, page_h, COLOR_CARD_BG);

        match browser.active_page {
            BrowserPage::Portal => render_portal(content_x, cur_y, content_w, page_h, fb),
            BrowserPage::Docs => render_docs(content_x, cur_y, content_w, fb),
            BrowserPage::StoreCatalog => render_web_store(content_x, cur_y, content_w, fb),
            BrowserPage::Releases => render_releases(content_x, cur_y, content_w, fb),
            BrowserPage::Search => render_search(content_x, cur_y, content_w, fb),
        }

        // 4. Status Bar
        let status_y = (win.y + win.height).saturating_sub(22);
        fb.fill_rect(content_x, status_y, content_w, 20, 0x00E4DFD5);
        fb.draw_string(content_x + 12, status_y + 3, browser.status_text, COLOR_CHARCOAL);
    }
}

fn draw_nav_btn(fb: &mut Framebuffer, x: usize, y: usize, label: &str) {
    fb.fill_rect(x, y, 22, 24, COLOR_WHITE);
    fb.draw_rect_outline(x, y, 22, 24, COLOR_OCHRE, 1);
    fb.draw_string(x + 7, y + 5, label, COLOR_CHARCOAL);
}

fn render_portal(px: usize, py: usize, pw: usize, _ph: usize, fb: &mut Framebuffer) {
    let center_x = px + pw / 2;
    let mut y = py + 20;

    fb.draw_zeroth_logo(center_x, y + 20, 20, COLOR_TERRACOTTA, COLOR_CHARCOAL);
    y += 50;

    let h1 = "ZeroWeb Explorer";
    fb.draw_string(center_x.saturating_sub((h1.len() * FONT_WIDTH) / 2), y, h1, COLOR_CHARCOAL);
    y += 20;

    let h2 = "Browse the open web and native .zero applications";
    fb.draw_string(center_x.saturating_sub((h2.len() * FONT_WIDTH) / 2), y, h2, COLOR_TERRACOTTA);
    y += 28;

    // Search Engine Bar (Clickable)
    let search_w = 420.min(pw.saturating_sub(40));
    let search_x = center_x.saturating_sub(search_w / 2);
    fb.fill_rect(search_x, y, search_w, 32, COLOR_WHITE);
    fb.draw_rect_outline(search_x, y, search_w, 32, COLOR_TERRACOTTA, 2);
    fb.draw_string(search_x + 16, y + 8, "[ Click here to Search ZeroWeb... ]", COLOR_MUTED);

    let sbtn_x = search_x + search_w.saturating_sub(84);
    fb.fill_rect(sbtn_x, y + 4, 76, 24, COLOR_CHARCOAL);
    fb.draw_string(sbtn_x + 14, y + 8, "Search", COLOR_WHITE);

    y += 48;

    // 4 Clickable Portal Cards
    let card_w = ((search_w.saturating_sub(20)) / 2).max(140);
    let card_h = 65;

    draw_card(fb, search_x, y, card_w, card_h, "1. Documentation", "System architecture & ABI");
    draw_card(fb, search_x + card_w + 20, y, card_w, card_h, "2. ZeroStore Web", "Browse verified .zero apps");

    y += card_h + 14;

    draw_card(fb, search_x, y, card_w, card_h, "3. Release Notes", "v0.1.0 UEFI bare-metal live");
    draw_card(fb, search_x + card_w + 20, y, card_w, card_h, "4. Search Engine", "Query web and packages");
}

fn render_docs(px: usize, py: usize, pw: usize, fb: &mut Framebuffer) {
    let mut y = py + 20;
    let card_x = px + 24;
    let card_w = pw.saturating_sub(48);

    fb.draw_string(card_x, y, "ZEROTH OS SYSTEM DOCUMENTATION", COLOR_CHARCOAL);
    fb.fill_rect(card_x, y + 18, card_w, 2, COLOR_TERRACOTTA);
    y += 28;

    fb.draw_string(card_x, y, "1. Kernel Privilege Model:", COLOR_TERRACOTTA);
    y += 18;
    fb.draw_string(card_x + 16, y, "- Ring 0 (Kernel Space): Memory paging, device drivers, IDT, scheduling", COLOR_CHARCOAL);
    y += 18;
    fb.draw_string(card_x + 16, y, "- Ring 3 (User Space): Isolated applications (.zero bundles)", COLOR_CHARCOAL);
    y += 28;

    fb.draw_string(card_x, y, "2. .zero Application Package Architecture:", COLOR_TERRACOTTA);
    y += 18;
    fb.draw_string(card_x + 16, y, "- Package Manifest: manifest.toml contains permissions and entry point", COLOR_CHARCOAL);
    y += 18;
    fb.draw_string(card_x + 16, y, "- Digital Signature: Ed25519 root trust prevents tampering", COLOR_CHARCOAL);
    y += 28;

    // Back button
    fb.fill_rect(card_x, y, 140, 26, COLOR_CHARCOAL);
    fb.draw_string(card_x + 14, y + 6, "< Back to Portal", COLOR_WHITE);
}

fn render_web_store(px: usize, py: usize, pw: usize, fb: &mut Framebuffer) {
    let mut y = py + 20;
    let card_x = px + 24;
    let card_w = pw.saturating_sub(48);

    fb.draw_string(card_x, y, "ZEROSTORE ONLINE WEB REPOSITORY", COLOR_CHARCOAL);
    fb.fill_rect(card_x, y + 18, card_w, 2, COLOR_OCHRE);
    y += 28;

    let items = [
        ("ZeroCalc.zero", "Scientific & Programmer Calculator", "INSTALLED"),
        ("ZeroPad.zero", "Syntax Code Editor for Rust/C/Python", "AVAILABLE"),
        ("ZeroPaint.zero", "Hand-Drawn Vector Art Canvas", "AVAILABLE"),
        ("ZeroMusic.zero", "Audio Player with FLAC/WAV Engine", "AVAILABLE"),
    ];

    for (name, desc, state) in items.iter() {
        fb.fill_rect(card_x, y, card_w, 38, COLOR_WHITE);
        fb.draw_rect_outline(card_x, y, card_w, 38, COLOR_OCHRE, 1);
        fb.draw_string(card_x + 12, y + 10, name, COLOR_TERRACOTTA);
        fb.draw_string(card_x + 160, y + 10, desc, COLOR_MUTED);

        let btn_color = if *state == "INSTALLED" { 0x00E4F0D0 } else { COLOR_TERRACOTTA };
        let text_color = if *state == "INSTALLED" { 0x002B6010 } else { COLOR_WHITE };
        fb.fill_rect(card_x + card_w.saturating_sub(90), y + 6, 80, 26, btn_color);
        fb.draw_string(card_x + card_w.saturating_sub(84), y + 10, state, text_color);

        y += 44;
    }

    // Back button
    fb.fill_rect(card_x, y, 140, 26, COLOR_CHARCOAL);
    fb.draw_string(card_x + 14, y + 6, "< Back to Portal", COLOR_WHITE);
}

fn render_releases(px: usize, py: usize, pw: usize, fb: &mut Framebuffer) {
    let mut y = py + 20;
    let card_x = px + 24;
    let card_w = pw.saturating_sub(48);

    fb.draw_string(card_x, y, "ZEROTH OS RELEASE NOTES — v0.1.0", COLOR_CHARCOAL);
    fb.fill_rect(card_x, y + 18, card_w, 2, COLOR_TERRACOTTA);
    y += 28;

    fb.draw_string(card_x, y, "* UEFI 64-bit Long Mode bootloader protocol verified.", COLOR_CHARCOAL);
    y += 20;
    fb.draw_string(card_x, y, "* 1280x720 Linear GOP Framebuffer engine enabled.", COLOR_CHARCOAL);
    y += 20;
    fb.draw_string(card_x, y, "* Flicker-Free Double-Buffered Cursor Overlay active.", COLOR_TERRACOTTA);
    y += 20;
    fb.draw_string(card_x, y, "* Native ZeroWeb Browser and ZeroTerminal operational.", COLOR_CHARCOAL);
    y += 28;

    fb.fill_rect(card_x, y, 140, 26, COLOR_CHARCOAL);
    fb.draw_string(card_x + 14, y + 6, "< Back to Portal", COLOR_WHITE);
}

fn render_search(px: usize, py: usize, pw: usize, fb: &mut Framebuffer) {
    let mut y = py + 20;
    let card_x = px + 24;
    let card_w = pw.saturating_sub(48);

    fb.draw_string(card_x, y, "SEARCH RESULTS: 'zeroth apps and features'", COLOR_CHARCOAL);
    fb.fill_rect(card_x, y + 18, card_w, 2, COLOR_TERRACOTTA);
    y += 28;

    fb.draw_string(card_x, y, "1. Getting Started with Zeroth OS (docs.zeroth.os)", COLOR_TERRACOTTA);
    y += 18;
    fb.draw_string(card_x + 12, y, "Official architectural manual and developer setup guide.", COLOR_MUTED);
    y += 26;

    fb.draw_string(card_x, y, "2. Browse Verified .zero Packages (store.zeroth.os)", COLOR_TERRACOTTA);
    y += 18;
    fb.draw_string(card_x + 12, y, "Explore software catalog with zero telemetry and instant sandbox.", COLOR_MUTED);
    y += 28;

    fb.fill_rect(card_x, y, 140, 26, COLOR_CHARCOAL);
    fb.draw_string(card_x + 14, y + 6, "< Back to Portal", COLOR_WHITE);
}

fn draw_card(fb: &mut Framebuffer, x: usize, y: usize, w: usize, h: usize, title: &str, desc: &str) {
    fb.fill_rect(x, y, w, h, COLOR_WHITE);
    fb.draw_rect_outline(x, y, w, h, COLOR_OCHRE, 1);
    fb.draw_string(x + 12, y + 10, title, COLOR_TERRACOTTA);
    fb.draw_string(x + 12, y + 30, desc, COLOR_CHARCOAL);
}

/// Handles click navigation inside the browser window
pub fn handle_browser_click(win: &Window, mx: usize, my: usize) -> bool {
    let content_x = win.x + 4;
    let cur_y = win.y + TITLEBAR_HEIGHT + 2;

    // Check Navigation Bar Buttons
    let btn_y = cur_y + 24 + 6;
    if my >= btn_y && my <= btn_y + 24 {
        // Back Button [ < ]
        if mx >= content_x + 10 && mx <= content_x + 32 {
            unsafe {
                BROWSER.active_page = BrowserPage::Portal;
                BROWSER.url_text = "https://zeroth.os/portal";
            }
            return true;
        }
        // Home Button [ H ]
        if mx >= content_x + 88 && mx <= content_x + 110 {
            unsafe {
                BROWSER.active_page = BrowserPage::Portal;
                BROWSER.url_text = "https://zeroth.os/portal";
            }
            return true;
        }
    }

    // Check Tab 1: Portal Tab
    if my >= cur_y && my <= cur_y + 24 {
        if mx >= content_x + 6 && mx <= content_x + 146 {
            unsafe {
                BROWSER.active_page = BrowserPage::Portal;
                BROWSER.url_text = "https://zeroth.os/portal";
            }
            return true;
        }
    }

    // Check Portal Page Cards Click
    unsafe {
        if BROWSER.active_page == BrowserPage::Portal {
            let center_x = content_x + (win.width.saturating_sub(8)) / 2;
            let search_w = 420.min(win.width.saturating_sub(48));
            let search_x = center_x.saturating_sub(search_w / 2);
            let search_y = cur_y + 60 + 20 + 20 + 28;

            // Search Bar Click
            if mx >= search_x && mx <= search_x + search_w && my >= search_y && my <= search_y + 32 {
                BROWSER.active_page = BrowserPage::Search;
                BROWSER.url_text = "https://zeroth.os/search?q=zeroth";
                return true;
            }

            let cards_y = search_y + 48;
            let card_w = ((search_w.saturating_sub(20)) / 2).max(140);
            let card_h = 65;

            // Card 1: Documentation
            if mx >= search_x && mx <= search_x + card_w && my >= cards_y && my <= cards_y + card_h {
                BROWSER.active_page = BrowserPage::Docs;
                BROWSER.url_text = "https://zeroth.os/docs/kernel";
                return true;
            }

            // Card 2: ZeroStore Web
            let card2_x = search_x + card_w + 20;
            if mx >= card2_x && mx <= card2_x + card_w && my >= cards_y && my <= cards_y + card_h {
                BROWSER.active_page = BrowserPage::StoreCatalog;
                BROWSER.url_text = "https://store.zeroth.os/catalog";
                return true;
            }

            let row2_y = cards_y + card_h + 14;
            // Card 3: Release Notes
            if mx >= search_x && mx <= search_x + card_w && my >= row2_y && my <= row2_y + card_h {
                BROWSER.active_page = BrowserPage::Releases;
                BROWSER.url_text = "https://zeroth.os/releases/v0.1.0";
                return true;
            }

            // Card 4: Search Engine
            if mx >= card2_x && mx <= card2_x + card_w && my >= row2_y && my <= row2_y + card_h {
                BROWSER.active_page = BrowserPage::Search;
                BROWSER.url_text = "https://zeroth.os/search";
                return true;
            }
        } else {
            // Inside Sub-pages: Check "< Back to Portal" button click
            let page_y = cur_y + 60;
            if my >= page_y + 120 && my <= page_y + 160 && mx >= content_x + 24 && mx <= content_x + 180 {
                BROWSER.active_page = BrowserPage::Portal;
                BROWSER.url_text = "https://zeroth.os/portal";
                return true;
            }
        }
    }

    false
}
