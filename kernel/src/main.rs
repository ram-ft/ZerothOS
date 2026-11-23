#![no_std]
#![no_main]

pub mod arch;
pub mod drivers;
pub mod gui;
pub mod mm;

use core::panic::PanicInfo;
use arch::serial::SERIAL;
use arch::gdt::init_gdt;
use arch::interrupts::init_idt;
use drivers::framebuffer::*;
use drivers::ps2::*;
use gui::window::{Window, AppType};
use gui::desktop::*;
use gui::apps::browser::{render_browser, handle_browser_click};
use gui::apps::terminal::{render_terminal, render_terminal_content, TERM};
use gui::apps::store::render_store;
use gui::apps::calculator::render_calc;

// --- Limine Bootloader Protocol Markers and Requests ---

#[used]
#[link_section = ".requests_start_marker"]
static START_MARKER: [u64; 4] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
];

#[used]
#[link_section = ".requests"]
static BASE_REVISION: [u64; 3] = [
    0xf9562b2d5c95a6c8,
    0x6a7b384944536bdc,
    2,
];

#[repr(C)]
pub struct LimineFramebuffer {
    pub address: *mut u32,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    pub unused: [u8; 7],
    pub edid_size: u64,
    pub edid: *mut u8,
    pub mode_count: u64,
    pub modes: *mut *mut (),
}

#[repr(C)]
pub struct LimineFramebufferResponse {
    pub revision: u64,
    pub framebuffer_count: u64,
    pub framebuffers: *mut *mut LimineFramebuffer,
}

#[repr(C)]
pub struct LimineFramebufferRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *mut LimineFramebufferResponse,
}

unsafe impl Sync for LimineFramebufferRequest {}

#[used]
#[link_section = ".requests"]
pub static mut FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x9d5827dcd881dd75,
        0xa3148604f6fab11b,
    ],
    revision: 0,
    response: core::ptr::null_mut(),
};

#[used]
#[link_section = ".requests_end_marker"]
static END_MARKER: [u64; 2] = [
    0xadc0e0531bb10d03,
    0x9572709f31764c62,
];

/// Zeroth OS Kernel Entry Point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    SERIAL.init();
    sprintln!("\n=======================================================");
    sprintln!("  ____                 _   _        ___  ____  ");
    sprintln!(" |__  / ___ _ __ ___ | |_| |__    / _ \\/ ___| ");
    sprintln!("   / / / _ \\ '__/ _ \\| __| '_ \\  | | | \\___ \\ ");
    sprintln!("  / /_|  __/ | | (_) | |_| | | | | |_| |___) |");
    sprintln!(" /____|\\___|_|  \\___/ \\__|_| |_|  \\___/|____/ ");
    sprintln!("=======================================================");
    sprintln!(" Zeroth OS Kernel v0.1.0 [Flicker-Free Interactive Desktop]");
    sprintln!("-------------------------------------------------------");

    init_gdt();
    init_idt();

    let mut fb_width = 1280;
    let mut fb_height = 720;

    unsafe {
        let resp_ptr = (&raw const FRAMEBUFFER_REQUEST).read().response;
        if !resp_ptr.is_null() && (*resp_ptr).framebuffer_count > 0 {
            let fb_ptr = *(*resp_ptr).framebuffers;
            if !fb_ptr.is_null() {
                let fb = &*fb_ptr;
                let fb_driver = &mut *(&raw mut FRAMEBUFFER);
                fb_driver.buffer = fb.address;
                fb_driver.width = fb.width as usize;
                fb_driver.height = fb.height as usize;
                fb_driver.pitch = (fb.pitch / 4) as usize;

                fb_width = fb_driver.width;
                fb_height = fb_driver.height;
                sprintln!("[GOP] Framebuffer active: {}x{}", fb_width, fb_height);
            }
        }
    }

    init_ps2(fb_width, fb_height);
    sprintln!("[PS2] Interactive Keyboard and Mouse initialized.");

    run_zeroth_desktop(fb_width, fb_height);
}

fn run_zeroth_desktop(screen_w: usize, screen_h: usize) -> ! {
    let fb = unsafe { &mut *(&raw mut FRAMEBUFFER) };

    // Window 0: ZeroWeb Browser
    let browser_w = 720.min(screen_w.saturating_sub(60));
    let browser_h = 460.min(screen_h.saturating_sub(90));
    let mut win_browser = Window::new(0, "ZeroWeb Browser", AppType::Browser, 30, 36, browser_w, browser_h);

    // Window 1: ZeroTerminal
    let term_w = 480.min(screen_w.saturating_sub(60));
    let term_h = 310.min(screen_h.saturating_sub(120));
    let mut win_term = Window::new(1, "ZeroTerminal (x86_64)", AppType::Terminal, screen_w.saturating_sub(term_w + 30), 65, term_w, term_h);

    // Window 2: ZeroStore
    let store_w = 580.min(screen_w.saturating_sub(80));
    let store_h = 400.min(screen_h.saturating_sub(110));
    let mut win_store = Window::new(2, "ZeroStore Marketplace", AppType::Store, 80, 50, store_w, store_h);
    win_store.is_minimized = true;

    // Window 3: ZeroCalc
    let calc_w = 260;
    let calc_h = 320;
    let mut win_calc = Window::new(3, "ZeroCalc", AppType::Calculator, screen_w / 2 - 130, 80, calc_w, calc_h);
    win_calc.is_minimized = true;

    let mut active_app = 0; // 0 = Browser, 1 = Terminal, 2 = Store, 3 = Calc

    // 1. Initial complete paint of Desktop & Windows
    render_all_desktop_and_windows(fb, &win_browser, &win_term, &win_store, &win_calc, active_app);

    // 2. Initial cursor draw with overlay save
    unsafe {
        fb.draw_flicker_free_cursor(MOUSE.x as usize, MOUSE.y as usize);
    }

    let mut dragging = false;
    let mut drag_win_id = 0;
    let mut drag_offset_x = 0;
    let mut drag_offset_y = 0;

    let mut last_mx = unsafe { MOUSE.x as usize };
    let mut last_my = unsafe { MOUSE.y as usize };

    loop {
        let (typed_char, mouse_moved, clicked) = poll_ps2_events(screen_w, screen_h);
        let mut full_redraw_needed = false;

        // 1. Handle Keyboard Typing
        if let Some(c) = typed_char {
            unsafe {
                TERM.push_char(c);
            }
            if win_term.is_minimized {
                win_term.is_minimized = false;
                full_redraw_needed = true;
            } else {
                // In-place terminal redraw (NO full screen blink!)
                fb.restore_cursor_overlay();
                render_terminal_content(&win_term, fb);
                unsafe {
                    fb.draw_flicker_free_cursor(MOUSE.x as usize, MOUSE.y as usize);
                }
            }
            active_app = 1;
        }

        let mx = unsafe { MOUSE.x as usize };
        let my = unsafe { MOUSE.y as usize };

        // 2. Handle Mouse Clicks
        if clicked {
            // Check Dock Clicks
            if let Some(dock_app) = get_dock_clicked_app(mx, my, screen_w, screen_h) {
                active_app = dock_app;
                match dock_app {
                    0 => win_browser.is_minimized = false,
                    1 => win_term.is_minimized = false,
                    2 => win_store.is_minimized = false,
                    3 => win_calc.is_minimized = false,
                    _ => {}
                }
                full_redraw_needed = true;
            }
            // Check Window Close [ X ] buttons
            else if !win_browser.is_minimized && win_browser.is_inside_close(mx, my) {
                win_browser.is_minimized = true;
                full_redraw_needed = true;
            } else if !win_term.is_minimized && win_term.is_inside_close(mx, my) {
                win_term.is_minimized = true;
                full_redraw_needed = true;
            } else if !win_store.is_minimized && win_store.is_inside_close(mx, my) {
                win_store.is_minimized = true;
                full_redraw_needed = true;
            } else if !win_calc.is_minimized && win_calc.is_inside_close(mx, my) {
                win_calc.is_minimized = true;
                full_redraw_needed = true;
            }
            // Check Browser Clicks (Links, Buttons, Tabs, Search)
            else if !win_browser.is_minimized && mx >= win_browser.x && mx <= win_browser.x + win_browser.width 
                    && my >= win_browser.y && my <= win_browser.y + win_browser.height {
                active_app = 0;
                if win_browser.is_inside_titlebar(mx, my) {
                    dragging = true;
                    drag_win_id = 0;
                    drag_offset_x = mx.saturating_sub(win_browser.x);
                    drag_offset_y = my.saturating_sub(win_browser.y);
                } else {
                    let navigated = handle_browser_click(&win_browser, mx, my);
                    if navigated {
                        full_redraw_needed = true;
                    }
                }
            }
            // Check Terminal Click
            else if !win_term.is_minimized && mx >= win_term.x && mx <= win_term.x + win_term.width 
                    && my >= win_term.y && my <= win_term.y + win_term.height {
                active_app = 1;
                if win_term.is_inside_titlebar(mx, my) {
                    dragging = true;
                    drag_win_id = 1;
                    drag_offset_x = mx.saturating_sub(win_term.x);
                    drag_offset_y = my.saturating_sub(win_term.y);
                }
            }
        }

        // 3. Handle Dragging
        unsafe {
            if !MOUSE.left_button && dragging {
                dragging = false;
            }
        }

        if dragging {
            let new_x = mx.saturating_sub(drag_offset_x);
            let new_y = my.saturating_sub(drag_offset_y);
            match drag_win_id {
                0 => { win_browser.x = new_x; win_browser.y = new_y; },
                1 => { win_term.x = new_x; win_term.y = new_y; },
                2 => { win_store.x = new_x; win_store.y = new_y; },
                3 => { win_calc.x = new_x; win_calc.y = new_y; },
                _ => {}
            }
            full_redraw_needed = true;
        }

        // 4. Handle Mouse Movement (FLICKER-FREE DIRTY RECT RESTORE!)
        if (mouse_moved || mx != last_mx || my != last_my) && !full_redraw_needed {
            // Restore ONLY the 16x16 under-cursor pixels!
            fb.restore_cursor_overlay();
            // Draw cursor at new position and save its background!
            fb.draw_flicker_free_cursor(mx, my);

            last_mx = mx;
            last_my = my;
        }

        // 5. Full Redraw when window structure changes
        if full_redraw_needed {
            render_all_desktop_and_windows(fb, &win_browser, &win_term, &win_store, &win_calc, active_app);
            fb.draw_flicker_free_cursor(mx, my);
            last_mx = mx;
            last_my = my;
        }

        core::hint::spin_loop();
    }
}

fn render_all_desktop_and_windows(
    fb: &mut Framebuffer,
    browser: &Window,
    term: &Window,
    store: &Window,
    calc: &Window,
    active_app: usize,
) {
    if !fb.is_valid() {
        return;
    }

    // 1. Wallpaper background
    draw_desktop_background(fb);

    // 2. Render Windows
    if !store.is_minimized {
        render_store(store, fb);
    }
    if !calc.is_minimized {
        render_calc(calc, fb);
    }

    // Secondary window first, focused window on top
    if active_app == 1 {
        if !browser.is_minimized { render_browser(browser, fb); }
        if !term.is_minimized { render_terminal(term, fb); }
    } else {
        if !term.is_minimized { render_terminal(term, fb); }
        if !browser.is_minimized { render_browser(browser, fb); }
    }

    // 3. Top System Bar
    let title = match active_app {
        0 => "ZeroWeb Browser",
        1 => "ZeroTerminal",
        2 => "ZeroStore",
        3 => "ZeroCalc",
        _ => "Desktop",
    };
    draw_top_bar(fb, title);

    // 4. Bottom Dock
    draw_bottom_dock(fb, active_app);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sprintln!("\n[KERNEL PANIC] Zeroth OS halted.");
    if let Some(location) = info.location() {
        sprintln!("Location: {}:{}", location.file(), location.line());
    }
    arch::hlt_loop();
}
