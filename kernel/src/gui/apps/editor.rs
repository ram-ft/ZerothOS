//! ZeroPad — Hand‑drawn style text editor for Zeroth OS
//! Features: cursor movement, insert/delete, line numbers, basic syntax highlighting,
//! and a simple in‑memory virtual file system for open/save.

use crate::drivers::framebuffer::*;
use crate::drivers::font::{FONT_WIDTH, FONT_HEIGHT};
use crate::gui::window::{Window, TITLEBAR_HEIGHT};
use core::str;

// -----------------------------------------------------------------------------
// Virtual File System (in‑memory map of file name -> content)
// -----------------------------------------------------------------------------
pub struct VirtualFile {
    pub name: [u8; 32],
    pub data: [u8; 4096], // 4 KB per file – sufficient for small source files
    pub len: usize,
}

impl VirtualFile {
    pub const fn empty() -> Self {
        Self { name: [0; 32], data: [0; 4096], len: 0 }
    }
    pub fn set_name(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let n = bytes.len().min(32);
        self.name[..n].copy_from_slice(&bytes[..n]);
    }
    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
    pub fn write(&mut self, src: &[u8]) {
        let n = src.len().min(self.data.len());
        self.data[..n].copy_from_slice(&src[..n]);
        self.len = n;
    }
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

// Very small FS – a fixed‑size array of files.
pub struct VirtualFS {
    pub files: [VirtualFile; 8],
    pub count: usize,
}

impl VirtualFS {
    pub const fn empty() -> Self {
        const EMPTY: VirtualFile = VirtualFile::empty();
        Self { files: [EMPTY; 8], count: 0 }
    }
    pub fn find_mut(&mut self, name: &str) -> Option<&mut VirtualFile> {
        for f in self.files.iter_mut().take(self.count) {
            let stored = str::from_utf8(&f.name).unwrap_or("");
            if stored.trim_end_matches('\0') == name {
                return Some(f);
            }
        }
        None
    }
    pub fn create(&mut self, name: &str) -> Option<&mut VirtualFile> {
        if self.count >= self.files.len() { return None; }
        let mut f = VirtualFile::empty();
        f.set_name(name);
        self.files[self.count] = f;
        let idx = self.count;
        self.count += 1;
        Some(&mut self.files[idx])
    }
    pub fn list_names(&self) -> heapless::Vec<&str, 8> {
        let mut v = heapless::Vec::new();
        for f in self.files.iter().take(self.count) {
            let s = str::from_utf8(&f.name).unwrap_or("");
            let name = s.trim_end_matches('\0');
            let _ = v.push(name);
        }
        v
    }
}

// -----------------------------------------------------------------------------
// Editor State
// -----------------------------------------------------------------------------
const MAX_LINES: usize = 200;
const MAX_LINE_LEN: usize = 256;

#[derive(Copy, Clone)]
pub struct EditLine {
    pub data: [u8; MAX_LINE_LEN],
    pub len: usize,
}

impl EditLine {
    pub const fn empty() -> Self { Self { data: [0; MAX_LINE_LEN], len: 0 } }
    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
    pub fn set(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let n = bytes.len().min(MAX_LINE_LEN);
        self.data[..n].copy_from_slice(&bytes[..n]);
        self.len = n;
    }
    pub fn insert_char(&mut self, pos: usize, c: u8) {
        if self.len >= MAX_LINE_LEN { return; }
        if pos > self.len { return; }
        for i in (pos..self.len).rev() {
            self.data[i + 1] = self.data[i];
        }
        self.data[pos] = c;
        self.len += 1;
    }
    pub fn delete_char(&mut self, pos: usize) {
        if pos >= self.len { return; }
        for i in pos..self.len - 1 {
            self.data[i] = self.data[i + 1];
        }
        self.len -= 1;
    }
}

pub struct EditorState {
    pub lines: [EditLine; MAX_LINES],
    pub line_count: usize,
    pub cursor_x: usize, // column in bytes
    pub cursor_y: usize, // line index
    pub filename: [u8; 32],
    pub dirty: bool,
    pub fs: VirtualFS,
}

impl EditorState {
    pub const fn new() -> Self {
        const EMPTY_LINE: EditLine = EditLine::empty();
        Self {
            lines: [EMPTY_LINE; MAX_LINES],
            line_count: 1,
            cursor_x: 0,
            cursor_y: 0,
            filename: [0; 32],
            dirty: false,
            fs: VirtualFS::empty(),
        }
    }
    fn current_line_mut(&mut self) -> &mut EditLine {
        &mut self.lines[self.cursor_y]
    }
    fn current_line(&self) -> &EditLine {
        &self.lines[self.cursor_y]
    }
    pub fn set_filename(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let n = bytes.len().min(32);
        self.filename[..n].copy_from_slice(&bytes[..n]);
    }
    pub fn filename_str(&self) -> &str {
        str::from_utf8(&self.filename).unwrap_or("").trim_end_matches('\0')
    }
    pub fn insert_char(&mut self, c: char) {
        if !c.is_ascii() { return; }
        let line = self.current_line_mut();
        line.insert_char(self.cursor_x, c as u8);
        self.cursor_x += 1;
        self.dirty = true;
    }
    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            let line = self.current_line_mut();
            line.delete_char(self.cursor_x - 1);
            self.cursor_x -= 1;
            self.dirty = true;
        } else if self.cursor_y > 0 {
            // Merge with previous line
            let cur_len = self.current_line().len;
            let prev_idx = self.cursor_y - 1;
            let prev_line = &mut self.lines[prev_idx];
            let new_x = prev_line.len;
            if prev_line.len + cur_len < MAX_LINE_LEN {
                // copy current line content
                let src = self.current_line().data;
                for i in 0..cur_len {
                    prev_line.data[prev_line.len + i] = src[i];
                }
                prev_line.len += cur_len;
                // shift all lines up
                for i in self.cursor_y..self.line_count - 1 {
                    self.lines[i] = self.lines[i + 1];
                }
                self.line_count -= 1;
                self.cursor_y -= 1;
                self.cursor_x = new_x;
                self.dirty = true;
            }
        }
    }
    pub fn newline(&mut self) {
        // Split current line at cursor
        let cur_line = self.current_line_mut();
        let after_len = cur_line.len - self.cursor_x;
        let mut new_line = EditLine::empty();
        if after_len > 0 {
            new_line.data[..after_len]
                .copy_from_slice(&cur_line.data[self.cursor_x..cur_line.len]);
            new_line.len = after_len;
        }
        cur_line.len = self.cursor_x;
        // shift lines down
        if self.line_count < MAX_LINES {
            for i in (self.cursor_y + 1..self.line_count).rev() {
                self.lines[i + 1] = self.lines[i];
            }
            self.lines[self.cursor_y + 1] = new_line;
            self.line_count += 1;
            self.cursor_y += 1;
            self.cursor_x = 0;
            self.dirty = true;
        }
    }
    pub fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len;
        }
    }
    pub fn move_right(&mut self) {
        let len = self.current_line().len;
        if self.cursor_x < len {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.line_count {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }
    pub fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            let line_len = self.lines[self.cursor_y].len;
            if self.cursor_x > line_len { self.cursor_x = line_len; }
        }
    }
    pub fn move_down(&mut self) {
        if self.cursor_y + 1 < self.line_count {
            self.cursor_y += 1;
            let line_len = self.lines[self.cursor_y].len;
            if self.cursor_x > line_len { self.cursor_x = line_len; }
        }
    }
    // ---------------------------------------------------------------------
    // File Operations (using the simple VirtualFS)
    // ---------------------------------------------------------------------
    pub fn load_file(&mut self, name: &str) {
        if let Some(f) = self.fs.find_mut(name) {
            // Reset editor buffer
            self.line_count = 0;
            let content = f.as_str();
            for (i, line) in content.lines().enumerate() {
                if i >= MAX_LINES { break; }
                self.lines[i].set(line);
                self.line_count += 1;
            }
            if self.line_count == 0 { self.lines[0] = EditLine::empty(); self.line_count = 1; }
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.set_filename(name);
            self.dirty = false;
        } else {
            // New file – start empty
            self.line_count = 1;
            self.lines[0] = EditLine::empty();
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.set_filename(name);
            self.dirty = true;
        }
    }
    pub fn save_file(&mut self) {
        let name = self.filename_str();
        if name.is_empty() { return; }
        let mut content = heapless::String::<2048>::new();
        for i in 0..self.line_count {
            let line = self.lines[i].as_str();
            let _ = write!(content, "{}\n", line);
        }
        if let Some(f) = self.fs.find_mut(name) {
            f.write(content.as_bytes());
        } else {
            if let Some(f) = self.fs.create(name) {
                f.write(content.as_bytes());
            }
        }
        self.dirty = false;
    }
}

// -----------------------------------------------------------------------------
// Rendering – Claude hand‑drawn UI
// -----------------------------------------------------------------------------
pub static mut EDITOR: EditorState = EditorState::new();

pub fn render_editor(win: &Window, fb: &mut Framebuffer) {
    if win.is_minimized { return; }
    win.draw_frame(fb);
    render_editor_content(win, fb);
}

pub fn render_editor_content(win: &Window, fb: &mut Framebuffer) {
    let content_x = win.x + 4;
    let content_y = win.y + TITLEBAR_HEIGHT + 2;
    let content_w = win.width.saturating_sub(8);
    let content_h = win.height.saturating_sub(TITLEBAR_HEIGHT + 6);

    // Background
    fb.fill_rect(content_x, content_y, content_w, content_h, COLOR_CARD_BG);

    // Line numbers area (wide enough for up to 999 lines)
    let ln_width = FONT_WIDTH * 4; // e.g., "999 "
    fb.fill_rect(content_x, content_y, ln_width, content_h, COLOR_TERRACOTTA);
    // Render line numbers
    let line_height = FONT_HEIGHT + 2;
    let mut ty = content_y + 4;
    unsafe {
        let ed = &EDITOR;
        let start = if ed.line_count > 0 { 0 } else { 0 };
        for i in start..ed.line_count {
            let num_str = {
                let mut buf = heapless::String::<8>::new();
                let _ = write!(buf, "{} ", i + 1);
                buf
            };
            fb.draw_string(content_x + 2, ty, &num_str, COLOR_WHITE);
            ty += line_height;
        }
    }

    // Render editor text area
    let text_x = content_x + ln_width + 4;
    let mut ty2 = content_y + 4;
    unsafe {
        let ed = &EDITOR;
        let visible = (content_h / line_height).min(ed.line_count);
        for i in 0..visible {
            let line = ed.lines[i].as_str();
            // Simple syntax highlight – color keywords for Rust
            let colored = syntax_highlight(line);
            fb.draw_string(text_x, ty2, &colored.0, colored.1);
            ty2 += line_height;
        }
        // Block cursor
        let cx = text_x + (ed.cursor_x * FONT_WIDTH);
        let cy = content_y + 4 + (ed.cursor_y * line_height);
        fb.fill_rect(cx, cy, FONT_WIDTH, FONT_HEIGHT, COLOR_OCHRE);
    }

    // Bottom status bar – file name and dirty flag
    let status_y = win.y + win.height - 20;
    fb.fill_rect(content_x, status_y, content_w, 20, COLOR_CHARCOAL);
    unsafe {
        let ed = &EDITOR;
        let name = ed.filename_str();
        let mut status = heapless::String::<64>::new();
        let _ = write!(status, "{} {}", name, if ed.dirty { "*" } else { "" });
        fb.draw_string(content_x + 8, status_y + 4, &status, COLOR_TERRACOTTA);
    }
}

// -----------------------------------------------------------------------------
// Very tiny syntax highlighter – only recognizes a few Rust keywords.
// Returns (text, color) where color is applied to the whole line.
// For a more granular approach you would need per‑character coloring, but that
// would increase framebuffer writes dramatically.
// -----------------------------------------------------------------------------
fn syntax_highlight(line: &str) -> (heapless::String<256>, u32) {
    // Keywords we color in terracotta, everything else white.
    const KEYWORDS: [&str; 7] = ["fn", "let", "mut", "pub", "struct", "impl", "enum"];
    let mut out = heapless::String::<256>::new();
    let mut color = COLOR_WHITE;
    for word in line.split_whitespace() {
        if KEYWORDS.contains(&word) {
            color = COLOR_TERRACOTTA;
            let _ = write!(out, "{} ", word);
            color = COLOR_WHITE; // reset for next token
        } else {
            let _ = write!(out, "{} ", word);
        }
    }
    (out, color)
}

// -----------------------------------------------------------------------------
// Input handling – called from the main event loop (similar to terminal).
// The caller must forward key events (typed_char) and special keys via a
// dedicated API. For brevity we expose a simple function here.
// -----------------------------------------------------------------------------
pub fn handle_editor_key(c: char) {
    unsafe {
        let ed = &mut EDITOR;
        match c {
            '\x08' => ed.backspace(), // Backspace
            '\n' => ed.newline(),
            '\x01' => ed.move_left(),   // Ctrl+A – left arrow
            '\x05' => ed.move_right(),  // Ctrl+E – right arrow
            // We could map actual arrow scancodes via the PS/2 driver; omitted.
            _ => ed.insert_char(c),
        }
    }
}

// -----------------------------------------------------------------------------
// Simple commands exposed via a pseudo‑command line (Ctrl+G to open command box)
// -----------------------------------------------------------------------------
pub fn editor_command(cmd: &str) {
    unsafe {
        let ed = &mut EDITOR;
        let parts: heapless::Vec<&str, 4> = cmd.split_whitespace().collect();
        if parts.is_empty() { return; }
        match parts[0] {
            "open" => {
                if parts.len() > 1 { ed.load_file(parts[1]); }
            }
            "save" => { ed.save_file(); }
            "new" => {
                ed.load_file("untitled.txt");
            }
            _ => {}
        }
    }
}
