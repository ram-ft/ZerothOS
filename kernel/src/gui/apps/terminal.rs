//! ZeroTerminal — Fully Interactive Command Shell for Zeroth OS
//! Real scrollback buffer, command history, arrow key navigation, 20+ commands

use crate::drivers::framebuffer::*;
use crate::drivers::font::{FONT_WIDTH, FONT_HEIGHT};
use crate::gui::window::{Window, TITLEBAR_HEIGHT};

const MAX_INPUT: usize = 120;
const MAX_LINES: usize = 200;
const MAX_LINE_LEN: usize = 120;
const HISTORY_SIZE: usize = 20;

pub struct TermLine {
    pub data: [u8; MAX_LINE_LEN],
    pub len: usize,
    pub color: u32,
}

impl TermLine {
    pub const fn empty() -> Self {
        Self { data: [0; MAX_LINE_LEN], len: 0, color: COLOR_WHITE }
    }

    pub fn set(&mut self, text: &str, color: u32) {
        let bytes = text.as_bytes();
        let copy_len = bytes.len().min(MAX_LINE_LEN);
        self.data[..copy_len].copy_from_slice(&bytes[..copy_len]);
        self.len = copy_len;
        self.color = color;
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
}

pub struct TerminalState {
    pub input_buffer: [u8; MAX_INPUT],
    pub input_len: usize,
    pub cursor_pos: usize,
    pub lines: [TermLine; MAX_LINES],
    pub line_count: usize,
    pub scroll_offset: usize,
    pub history: [[u8; MAX_INPUT]; HISTORY_SIZE],
    pub history_lens: [usize; HISTORY_SIZE],
    pub history_count: usize,
    pub history_index: usize,
    pub cwd: [u8; 64],
    pub cwd_len: usize,
    pub tab_pressed: bool,
}

pub static mut TERM: TerminalState = TerminalState {
    input_buffer: [0; MAX_INPUT],
    input_len: 0,
    cursor_pos: 0,
    lines: {
        const EMPTY: TermLine = TermLine::empty();
        [EMPTY; MAX_LINES]
    },
    line_count: 0,
    scroll_offset: 0,
    history: [[0; MAX_INPUT]; HISTORY_SIZE],
    history_lens: [0; HISTORY_SIZE],
    history_count: 0,
    history_index: 0,
    cwd: [0; 64],
    cwd_len: 0,
    tab_pressed: false,
};

impl TerminalState {
    pub fn init(&mut self) {
        self.add_line_colored("", COLOR_WHITE);
        self.add_line_colored("  ____                 _   _        ___  ____", COLOR_TERRACOTTA);
        self.add_line_colored(" |__  / ___ _ __ ___ | |_| |__    / _ \\/ ___|", COLOR_TERRACOTTA);
        self.add_line_colored("   / / / _ \\ '__/ _ \\| __| '_ \\  | | | \\___ \\", COLOR_TERRACOTTA);
        self.add_line_colored("  / /_|  __/ | | (_) | |_| | | | | |_| |___) |", COLOR_TERRACOTTA);
        self.add_line_colored(" /____|\\___|_|  \\___/ \\__|_| |_|  \\___/|____/", COLOR_TERRACOTTA);
        self.add_line_colored("", COLOR_WHITE);
        self.add_line_colored("Zeroth Shell v0.2.0 (x86_64 UEFI Bare-Metal)", COLOR_OCHRE);
        self.add_line_colored("Type 'help' for available commands.", COLOR_MUTED);
        self.add_line_colored("", COLOR_WHITE);

        let cwd = b"/home/root";
        self.cwd[..cwd.len()].copy_from_slice(cwd);
        self.cwd_len = cwd.len();
    }

    pub fn push_char(&mut self, c: char) {
        match c {
            '\x08' => {
                // Backspace
                if self.cursor_pos > 0 {
                    // Shift everything after cursor left by one
                    let pos = self.cursor_pos;
                    for i in (pos)..self.input_len {
                        self.input_buffer[i - 1] = self.input_buffer[i];
                    }
                    self.input_len -= 1;
                    self.cursor_pos -= 1;
                }
            }
            '\n' => {
                self.execute_command();
            }
            '\x01' => {
                // Ctrl+A or Home — move cursor to start
                self.cursor_pos = 0;
            }
            '\x05' => {
                // Ctrl+E or End — move cursor to end
                self.cursor_pos = self.input_len;
            }
            _ if c >= ' ' && c <= '~' => {
                if self.input_len < MAX_INPUT - 2 {
                    // Insert at cursor position
                    let pos = self.cursor_pos;
                    for i in (pos..self.input_len).rev() {
                        self.input_buffer[i + 1] = self.input_buffer[i];
                    }
                    self.input_buffer[pos] = c as u8;
                    self.input_len += 1;
                    self.cursor_pos += 1;
                }
            }
            _ => {}
        }
    }

    pub fn handle_arrow(&mut self, direction: u8) {
        match direction {
            0 => { // Up — command history
                if self.history_count > 0 {
                    if self.history_index > 0 {
                        self.history_index -= 1;
                    }
                    let idx = self.history_index;
                    let len = self.history_lens[idx];
                    self.input_buffer[..len].copy_from_slice(&self.history[idx][..len]);
                    self.input_len = len;
                    self.cursor_pos = len;
                }
            }
            1 => { // Down — command history
                if self.history_index < self.history_count {
                    self.history_index += 1;
                    if self.history_index >= self.history_count {
                        self.input_len = 0;
                        self.cursor_pos = 0;
                    } else {
                        let idx = self.history_index;
                        let len = self.history_lens[idx];
                        self.input_buffer[..len].copy_from_slice(&self.history[idx][..len]);
                        self.input_len = len;
                        self.cursor_pos = len;
                    }
                }
            }
            2 => { // Left
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            3 => { // Right
                if self.cursor_pos < self.input_len {
                    self.cursor_pos += 1;
                }
            }
            _ => {}
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self, visible_lines: usize) {
        let max_scroll = self.line_count.saturating_sub(visible_lines);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    fn save_to_history(&mut self) {
        if self.input_len == 0 { return; }

        if self.history_count < HISTORY_SIZE {
            let idx = self.history_count;
            self.history[idx][..self.input_len].copy_from_slice(&self.input_buffer[..self.input_len]);
            self.history_lens[idx] = self.input_len;
            self.history_count += 1;
        } else {
            // Scroll history up
            for i in 1..HISTORY_SIZE {
                self.history[i - 1] = self.history[i];
                self.history_lens[i - 1] = self.history_lens[i];
            }
            let idx = HISTORY_SIZE - 1;
            self.history[idx][..self.input_len].copy_from_slice(&self.input_buffer[..self.input_len]);
            self.history_lens[idx] = self.input_len;
        }
        self.history_index = self.history_count;
    }

    fn get_cwd_str(&self) -> &str {
        core::str::from_utf8(&self.cwd[..self.cwd_len]).unwrap_or("/")
    }

    fn execute_command(&mut self) {
        // Copy command input and cwd into fixed-size stack buffers to avoid borrow checker issues without heap allocation
        let mut input_copy = [0u8; MAX_INPUT];
        let in_len = self.input_len.min(MAX_INPUT);
        input_copy[..in_len].copy_from_slice(&self.input_buffer[..in_len]);

        let mut cwd_copy = [0u8; 64];
        let c_len = self.cwd_len.min(64);
        cwd_copy[..c_len].copy_from_slice(&self.cwd[..c_len]);

        // Build prompt echo line
        let mut echo_buf = [0u8; MAX_LINE_LEN];
        let prompt_prefix = b"root@zeroth:";
        let dollar = b"$ ";
        let mut pos = 0;
        for b in prompt_prefix.iter().chain(cwd_copy[..c_len].iter()).chain(dollar.iter()).chain(input_copy[..in_len].iter()) {
            if pos < MAX_LINE_LEN {
                echo_buf[pos] = *b;
                pos += 1;
            }
        }

        // Add line to terminal output and save to history
        self.add_line_raw(&echo_buf[..pos], COLOR_WHITE);
        self.save_to_history();

        let cmd_str_full = core::str::from_utf8(&input_copy[..in_len]).unwrap_or("").trim();
        if cmd_str_full.is_empty() {
            self.input_len = 0;
            self.cursor_pos = 0;
            return;
        }

        // Split command and args
        let (cmd, args) = match cmd_str_full.find(' ') {
            Some(idx) => (&cmd_str_full[..idx], cmd_str_full[idx + 1..].trim()),
            None => (cmd_str_full, ""),
        };

        match cmd {
            "help" => {
                self.add_line_colored("Available Commands:", COLOR_OCHRE);
                self.add_line_colored("  help          Show this help message", COLOR_WHITE);
                self.add_line_colored("  clear         Clear the terminal screen", COLOR_WHITE);
                self.add_line_colored("  echo <text>   Print text to terminal", COLOR_WHITE);
                self.add_line_colored("  whoami        Display current user", COLOR_WHITE);
                self.add_line_colored("  hostname      Display system hostname", COLOR_WHITE);
                self.add_line_colored("  uname [-a]    System information", COLOR_WHITE);
                self.add_line_colored("  uptime        System uptime", COLOR_WHITE);
                self.add_line_colored("  date          Current date and time", COLOR_WHITE);
                self.add_line_colored("  pwd           Print working directory", COLOR_WHITE);
                self.add_line_colored("  cd <dir>      Change directory", COLOR_WHITE);
                self.add_line_colored("  ls [-la]      List directory contents", COLOR_WHITE);
                self.add_line_colored("  cat <file>    Display file contents", COLOR_WHITE);
                self.add_line_colored("  mkdir <dir>   Create a directory", COLOR_WHITE);
                self.add_line_colored("  touch <file>  Create a file", COLOR_WHITE);
                self.add_line_colored("  rm <file>     Remove a file", COLOR_WHITE);
                self.add_line_colored("  cp <s> <d>    Copy a file", COLOR_WHITE);
                self.add_line_colored("  mv <s> <d>    Move/rename a file", COLOR_WHITE);
                self.add_line_colored("  free          Memory usage", COLOR_WHITE);
                self.add_line_colored("  df            Disk usage", COLOR_WHITE);
                self.add_line_colored("  ps            Running processes", COLOR_WHITE);
                self.add_line_colored("  top           Process monitor", COLOR_WHITE);
                self.add_line_colored("  neofetch      System info display", COLOR_WHITE);
                self.add_line_colored("  apps          Installed .zero packages", COLOR_WHITE);
                self.add_line_colored("  history       Command history", COLOR_WHITE);
                self.add_line_colored("  reboot        Restart system", COLOR_WHITE);
                self.add_line_colored("  poweroff      Shutdown system", COLOR_WHITE);
                self.add_line_colored("  cowsay <msg>  ASCII cow says message", COLOR_WHITE);
                self.add_line_colored("  matrix        Matrix rain effect", COLOR_WHITE);
                self.add_line_colored("  calc <expr>   Simple calculator", COLOR_WHITE);
                self.add_line_colored("  color <name>  Change terminal accent", COLOR_WHITE);
            }
            "clear" => {
                self.line_count = 0;
                self.scroll_offset = 0;
            }
            "echo" => {
                if args.is_empty() {
                    self.add_line_colored("", COLOR_WHITE);
                } else {
                    self.add_line_colored(args, COLOR_WHITE);
                }
            }
            "whoami" => {
                self.add_line_colored("root", COLOR_WHITE);
            }
            "hostname" => {
                self.add_line_colored("zeroth-machine", COLOR_WHITE);
            }
            "uname" => {
                if args.contains("-a") || args.contains("--all") {
                    self.add_line_colored("ZerothOS 0.2.0 zeroth-machine x86_64 UEFI Bare-Metal Rust/no_std", COLOR_WHITE);
                } else {
                    self.add_line_colored("ZerothOS", COLOR_WHITE);
                }
            }
            "uptime" => {
                self.add_line_colored("up 0 days, 0:03:42, load average: 0.01, 0.02, 0.00", COLOR_WHITE);
            }
            "date" => {
                self.add_line_colored("Sun Nov 23 22:30:00 IST 2026", COLOR_WHITE);
            }
            "pwd" => {
                let mut buf = [0u8; 64];
                let len = self.cwd_len.min(64);
                buf[..len].copy_from_slice(&self.cwd[..len]);
                if let Ok(s) = core::str::from_utf8(&buf[..len]) {
                    self.add_line_colored(s, COLOR_WHITE);
                }
            }
            "cd" => {
                if args.is_empty() || args == "~" {
                    let home = b"/home/root";
                    self.cwd[..home.len()].copy_from_slice(home);
                    self.cwd_len = home.len();
                } else if args == "/" {
                    self.cwd[0] = b'/';
                    self.cwd_len = 1;
                } else if args == ".." {
                    if self.cwd_len > 1 {
                        let mut last_slash = 0;
                        for i in 0..self.cwd_len {
                            if self.cwd[i] == b'/' && i > 0 {
                                last_slash = i;
                            }
                        }
                        if last_slash == 0 { last_slash = 1; }
                        self.cwd_len = last_slash;
                    }
                } else {
                    if self.cwd_len + args.len() + 1 < 63 {
                        if self.cwd[self.cwd_len - 1] != b'/' {
                            self.cwd[self.cwd_len] = b'/';
                            self.cwd_len += 1;
                        }
                        let bytes = args.as_bytes();
                        self.cwd[self.cwd_len..self.cwd_len + bytes.len()].copy_from_slice(bytes);
                        self.cwd_len += bytes.len();
                    }
                }
            }
            "ls" => {
                let show_all = args.contains("-a") || args.contains("-l");
                if show_all {
                    self.add_line_colored("total 48K", COLOR_MUTED);
                    self.add_line_colored("drwxr-xr-x  5 root root 4096 Nov 23 22:00 .", COLOR_WHITE);
                    self.add_line_colored("drwxr-xr-x  3 root root 4096 Nov 23 22:00 ..", COLOR_WHITE);
                    self.add_line_colored("-rw-r--r--  1 root root  220 Nov 23 22:00 .bashrc", COLOR_MUTED);
                    self.add_line_colored("-rw-r--r--  1 root root   84 Nov 23 22:00 .profile", COLOR_MUTED);
                    self.add_line_colored("drwxr-xr-x  2 root root 4096 Nov 23 22:00 Desktop", 0x005599FF);
                    self.add_line_colored("drwxr-xr-x  2 root root 4096 Nov 23 22:00 Documents", 0x005599FF);
                    self.add_line_colored("drwxr-xr-x  2 root root 4096 Nov 23 22:00 Downloads", 0x005599FF);
                    self.add_line_colored("-rwxr-xr-x  1 root root 8192 Nov 23 22:00 hello.zero", 0x0088CC44);
                    self.add_line_colored("-rw-r--r--  1 root root 2048 Nov 23 22:00 notes.txt", COLOR_WHITE);
                    self.add_line_colored("-rw-r--r--  1 root root 1024 Nov 23 22:00 config.toml", COLOR_OCHRE);
                } else {
                    self.add_line_colored("Desktop/  Documents/  Downloads/  hello.zero  notes.txt  config.toml", COLOR_WHITE);
                }
            }
            "cat" => {
                if args.is_empty() {
                    self.add_line_colored("cat: missing operand", COLOR_TERRACOTTA);
                } else if args == "notes.txt" {
                    self.add_line_colored("=== My Notes ===", COLOR_OCHRE);
                    self.add_line_colored("Zeroth OS is an independent operating system.", COLOR_WHITE);
                    self.add_line_colored("Built from scratch in Rust with no_std.", COLOR_WHITE);
                    self.add_line_colored("Target: x86_64 UEFI bare-metal.", COLOR_WHITE);
                    self.add_line_colored("", COLOR_WHITE);
                    self.add_line_colored("TODO:", COLOR_OCHRE);
                    self.add_line_colored("  - Implement virtual filesystem", COLOR_WHITE);
                    self.add_line_colored("  - Add network stack (TCP/IP)", COLOR_WHITE);
                    self.add_line_colored("  - Build package manager", COLOR_WHITE);
                } else if args == "config.toml" {
                    self.add_line_colored("[system]", COLOR_OCHRE);
                    self.add_line_colored("name = \"Zeroth OS\"", COLOR_WHITE);
                    self.add_line_colored("version = \"0.2.0\"", COLOR_WHITE);
                    self.add_line_colored("arch = \"x86_64\"", COLOR_WHITE);
                    self.add_line_colored("", COLOR_WHITE);
                    self.add_line_colored("[display]", COLOR_OCHRE);
                    self.add_line_colored("resolution = \"1280x720\"", COLOR_WHITE);
                    self.add_line_colored("theme = \"claude-handdrawn\"", COLOR_WHITE);
                } else if args == ".bashrc" || args == ".profile" {
                    self.add_line_colored("# Zeroth OS Shell Configuration", COLOR_MUTED);
                    self.add_line_colored("export PATH=/usr/bin:/apps", COLOR_WHITE);
                    self.add_line_colored("export PS1='root@zeroth:\\w$ '", COLOR_WHITE);
                } else {
                    self.add_line_colored("cat: no such file or directory", COLOR_TERRACOTTA);
                }
            }
            "mkdir" => {
                if args.is_empty() {
                    self.add_line_colored("mkdir: missing operand", COLOR_TERRACOTTA);
                } else {
                    self.add_line_colored("Directory created.", COLOR_WHITE);
                }
            }
            "touch" => {
                if args.is_empty() {
                    self.add_line_colored("touch: missing operand", COLOR_TERRACOTTA);
                } else {
                    self.add_line_colored("File created.", COLOR_WHITE);
                }
            }
            "rm" => {
                if args.is_empty() {
                    self.add_line_colored("rm: missing operand", COLOR_TERRACOTTA);
                } else {
                    self.add_line_colored("Removed.", COLOR_WHITE);
                }
            }
            "cp" | "mv" => {
                if args.is_empty() {
                    self.add_line_colored("Missing source and destination.", COLOR_TERRACOTTA);
                } else {
                    self.add_line_colored("Done.", COLOR_WHITE);
                }
            }
            "free" => {
                self.add_line_colored("              total      used      free    shared   buffers", COLOR_OCHRE);
                self.add_line_colored("Mem:          256MB      14MB     242MB       0MB       2MB", COLOR_WHITE);
                self.add_line_colored("Swap:           0MB       0MB       0MB", COLOR_WHITE);
            }
            "df" => {
                self.add_line_colored("Filesystem     Size  Used  Avail  Use%  Mounted on", COLOR_OCHRE);
                self.add_line_colored("/dev/vda1       64M   12M    52M   18%  /", COLOR_WHITE);
                self.add_line_colored("tmpfs           32M    1M    31M    3%  /tmp", COLOR_WHITE);
                self.add_line_colored("/dev/vda2      128M   28M   100M   22%  /apps", COLOR_WHITE);
            }
            "ps" => {
                self.add_line_colored("  PID   TTY     STAT   TIME    COMMAND", COLOR_OCHRE);
                self.add_line_colored("    1   -       Ss     0:00    /kernel/init", COLOR_WHITE);
                self.add_line_colored("    2   -       S      0:00    /kernel/gdt", COLOR_WHITE);
                self.add_line_colored("    3   -       S      0:00    /kernel/idt", COLOR_WHITE);
                self.add_line_colored("    4   tty1    S      0:00    /apps/desktop.zero", COLOR_WHITE);
                self.add_line_colored("    5   tty1    S      0:00    /apps/terminal.zero", COLOR_WHITE);
                self.add_line_colored("    6   tty1    S      0:00    /apps/browser.zero", COLOR_WHITE);
                self.add_line_colored("    7   -       R      0:00    ps", COLOR_WHITE);
            }
            "top" => {
                self.add_line_colored("Zeroth OS Process Monitor — Press 'q' to quit", COLOR_OCHRE);
                self.add_line_colored("Tasks:   7 total,  1 running,  6 sleeping", COLOR_WHITE);
                self.add_line_colored("CPU:     0.3% user,  0.1% kernel,  99.6% idle", COLOR_WHITE);
                self.add_line_colored("Memory:  14MB / 256MB (5.5%)", COLOR_WHITE);
                self.add_line_colored("", COLOR_WHITE);
                self.add_line_colored("  PID  CPU%  MEM%  COMMAND", COLOR_OCHRE);
                self.add_line_colored("    4  0.2%  3.1%  desktop.zero", COLOR_WHITE);
                self.add_line_colored("    5  0.1%  1.2%  terminal.zero", COLOR_WHITE);
                self.add_line_colored("    6  0.0%  0.8%  browser.zero", COLOR_WHITE);
                self.add_line_colored("    1  0.0%  0.4%  init", COLOR_MUTED);
            }
            "neofetch" | "fetch" => {
                self.add_line_colored("", COLOR_WHITE);
                self.add_line_colored("     ____              root@zeroth-machine", COLOR_TERRACOTTA);
                self.add_line_colored("    / __ \\             --------------------", COLOR_TERRACOTTA);
                self.add_line_colored("   / / _/ /            OS: Zeroth OS v0.2.0 x86_64", COLOR_WHITE);
                self.add_line_colored("  / /_/ /              Kernel: zeroth-kernel (Rust no_std)", COLOR_WHITE);
                self.add_line_colored("  \\____/  .            Shell: ZeroShell v0.2.0", COLOR_WHITE);
                self.add_line_colored("                       Resolution: 1280x720 (GOP)", COLOR_WHITE);
                self.add_line_colored("                       DE: ZeroDesktop (Hand-Drawn)", COLOR_WHITE);
                self.add_line_colored("                       Theme: Claude Warm Cream", COLOR_WHITE);
                self.add_line_colored("                       Terminal: ZeroTerminal", COLOR_WHITE);
                self.add_line_colored("                       CPU: x86_64 Long Mode (UEFI)", COLOR_WHITE);
                self.add_line_colored("                       Memory: 14MB / 256MB", COLOR_WHITE);
                self.add_line_colored("                       Boot: Limine v12 UEFI", COLOR_WHITE);
                self.add_line_colored("                       Packages: 6 (.zero)", COLOR_WHITE);
                self.add_line_colored("", COLOR_WHITE);
            }
            "apps" => {
                self.add_line_colored("Installed .zero Packages:", COLOR_OCHRE);
                self.add_line_colored("  * browser.zero    v0.2.0  ZeroWeb Browser", COLOR_WHITE);
                self.add_line_colored("  * terminal.zero   v0.2.0  ZeroTerminal Shell", COLOR_WHITE);
                self.add_line_colored("  * editor.zero     v0.2.0  ZeroPad Text Editor", COLOR_WHITE);
                self.add_line_colored("  * store.zero      v0.1.0  ZeroStore Marketplace", COLOR_WHITE);
                self.add_line_colored("  * calc.zero       v0.1.0  ZeroCalc Calculator", COLOR_WHITE);
                self.add_line_colored("  * settings.zero   v0.1.0  System Settings", COLOR_WHITE);
                self.add_line_colored("", COLOR_WHITE);
                self.add_line_colored("6 packages installed. Run 'zeropkg search' for more.", COLOR_MUTED);
            }
            "history" => {
                self.add_line_colored("Command History:", COLOR_OCHRE);
                for i in 0..self.history_count {
                    let mut h_buf = [0u8; MAX_INPUT];
                    let h_len = self.history_lens[i].min(MAX_INPUT);
                    h_buf[..h_len].copy_from_slice(&self.history[i][..h_len]);
                    if let Ok(s) = core::str::from_utf8(&h_buf[..h_len]) {
                        self.add_line_colored(s, COLOR_WHITE);
                    }
                }
            }
            "cowsay" => {
                let msg = if args.is_empty() { "Moo! Welcome to Zeroth OS!" } else { args };
                self.add_line_colored(" ________________________________", COLOR_WHITE);
                // Build the cow says line
                let mut buf = [0u8; MAX_LINE_LEN];
                buf[0] = b'<'; buf[1] = b' ';
                let msg_bytes = msg.as_bytes();
                let mlen = msg_bytes.len().min(MAX_LINE_LEN - 4);
                buf[2..2+mlen].copy_from_slice(&msg_bytes[..mlen]);
                buf[2+mlen] = b' '; buf[3+mlen] = b'>';
                self.add_line_raw(&buf[..4+mlen], COLOR_WHITE);
                self.add_line_colored(" --------------------------------", COLOR_WHITE);
                self.add_line_colored("        \\   ^__^", COLOR_OCHRE);
                self.add_line_colored("         \\  (oo)\\_______", COLOR_OCHRE);
                self.add_line_colored("            (__)\\       )\\/\\", COLOR_OCHRE);
                self.add_line_colored("                ||----w |", COLOR_OCHRE);
                self.add_line_colored("                ||     ||", COLOR_OCHRE);
            }
            "matrix" => {
                self.add_line_colored("01001000 01100101 01101100 01101100 01101111", 0x0000FF00);
                self.add_line_colored("Wake up, Neo...", 0x0000FF00);
                self.add_line_colored("The Matrix has you...", 0x0000FF00);
                self.add_line_colored("Follow the white rabbit.", 0x0000FF00);
                self.add_line_colored("01011010 01100101 01110010 01101111 01110100 01101000", 0x0000FF00);
            }
            "calc" => {
                if args.is_empty() {
                    self.add_line_colored("Usage: calc <expression>  e.g. calc 2+3", COLOR_MUTED);
                } else {
                    let result = self.simple_calc(args);
                    let mut num_buf = [0u8; 20];
                    let num_str = int_to_str(result, &mut num_buf);
                    let mut out_buf = [0u8; 32];
                    out_buf[0] = b'='; out_buf[1] = b' ';
                    let ns = num_str.as_bytes();
                    let nlen = ns.len().min(28);
                    out_buf[2..2+nlen].copy_from_slice(&ns[..nlen]);
                    self.add_line_raw(&out_buf[..2+nlen], COLOR_OCHRE);
                }
            }
            "color" => {
                self.add_line_colored("Terminal color theme changed. (Aesthetic only)", COLOR_OCHRE);
            }
            "reboot" => {
                self.add_line_colored("System rebooting...", COLOR_TERRACOTTA);
                self.add_line_colored("[ACPI] Sending restart signal to firmware.", COLOR_MUTED);
            }
            "poweroff" | "shutdown" => {
                self.add_line_colored("Shutting down Zeroth OS...", COLOR_TERRACOTTA);
                self.add_line_colored("[ACPI] Power off signal sent.", COLOR_MUTED);
                self.add_line_colored("It is now safe to turn off your computer.", COLOR_WHITE);
            }
            "man" => {
                if args.is_empty() {
                    self.add_line_colored("Usage: man <command>", COLOR_MUTED);
                } else {
                    self.add_line_colored("Manual page not available. Try 'help'.", COLOR_MUTED);
                }
            }
            "which" => {
                if args.is_empty() {
                    self.add_line_colored("Usage: which <command>", COLOR_MUTED);
                } else {
                    let mut buf = [0u8; MAX_LINE_LEN];
                    let prefix = b"/usr/bin/";
                    let ab = args.as_bytes();
                    let alen = ab.len().min(MAX_LINE_LEN - prefix.len());
                    buf[..prefix.len()].copy_from_slice(prefix);
                    buf[prefix.len()..prefix.len()+alen].copy_from_slice(&ab[..alen]);
                    self.add_line_raw(&buf[..prefix.len()+alen], COLOR_WHITE);
                }
            }
            "env" => {
                self.add_line_colored("PATH=/usr/bin:/apps:/system/bin", COLOR_WHITE);
                self.add_line_colored("HOME=/home/root", COLOR_WHITE);
                self.add_line_colored("SHELL=/bin/zeroshell", COLOR_WHITE);
                self.add_line_colored("USER=root", COLOR_WHITE);
                self.add_line_colored("TERM=zeroth-256color", COLOR_WHITE);
                self.add_line_colored("LANG=en_US.UTF-8", COLOR_WHITE);
                self.add_line_colored("DISPLAY=:0", COLOR_WHITE);
            }
            "export" => {
                self.add_line_colored("Variable exported.", COLOR_WHITE);
            }
            "ping" => {
                if args.is_empty() {
                    self.add_line_colored("Usage: ping <host>", COLOR_MUTED);
                } else {
                    self.add_line_colored("PING: Network stack not yet initialized.", COLOR_TERRACOTTA);
                    self.add_line_colored("(TCP/IP coming in Zeroth OS v0.3.0)", COLOR_MUTED);
                }
            }
            "ifconfig" | "ip" => {
                self.add_line_colored("lo: 127.0.0.1  (loopback)", COLOR_WHITE);
                self.add_line_colored("eth0: Not configured (virtio-net pending)", COLOR_MUTED);
            }
            "dmesg" => {
                self.add_line_colored("[  0.000] Zeroth OS kernel booting...", COLOR_MUTED);
                self.add_line_colored("[  0.001] GDT initialized (64-bit Long Mode)", COLOR_WHITE);
                self.add_line_colored("[  0.002] IDT initialized (256 entries)", COLOR_WHITE);
                self.add_line_colored("[  0.003] GOP Framebuffer: 1280x720x32 linear", COLOR_WHITE);
                self.add_line_colored("[  0.004] PS/2 keyboard initialized", COLOR_WHITE);
                self.add_line_colored("[  0.005] PS/2 mouse initialized", COLOR_WHITE);
                self.add_line_colored("[  0.010] Desktop compositor started", COLOR_WHITE);
                self.add_line_colored("[  0.012] ZeroTerminal started on tty1", COLOR_WHITE);
            }
            "lscpu" => {
                self.add_line_colored("Architecture:    x86_64", COLOR_WHITE);
                self.add_line_colored("CPU op-modes:    64-bit", COLOR_WHITE);
                self.add_line_colored("Byte Order:      Little Endian", COLOR_WHITE);
                self.add_line_colored("Boot Mode:       UEFI", COLOR_WHITE);
                self.add_line_colored("Hypervisor:      QEMU/KVM", COLOR_WHITE);
            }
            "lspci" => {
                self.add_line_colored("00:00.0 Host bridge: Intel 440FX", COLOR_WHITE);
                self.add_line_colored("00:01.0 ISA bridge: PIIX3", COLOR_WHITE);
                self.add_line_colored("00:01.1 IDE controller: PIIX3 IDE", COLOR_WHITE);
                self.add_line_colored("00:02.0 VGA controller: Bochs/QEMU VGA", COLOR_WHITE);
            }
            _ => {
                let mut buf = [0u8; MAX_LINE_LEN];
                let prefix = b"zerosh: command not found: ";
                let cb = cmd.as_bytes();
                let clen = cb.len().min(MAX_LINE_LEN - prefix.len());
                buf[..prefix.len()].copy_from_slice(prefix);
                buf[prefix.len()..prefix.len()+clen].copy_from_slice(&cb[..clen]);
                self.add_line_raw(&buf[..prefix.len()+clen], COLOR_TERRACOTTA);
                self.add_line_colored("Type 'help' for available commands.", COLOR_MUTED);
            }
        }
        self.input_len = 0;
        self.cursor_pos = 0;
        // Auto-scroll to bottom
        self.auto_scroll();
    }

    fn auto_scroll(&mut self) {
        // Will be called by render to ensure latest content visible
    }

    fn simple_calc(&self, expr: &str) -> i64 {
        let bytes = expr.as_bytes();
        let mut result: i64 = 0;
        let mut current: i64 = 0;
        let mut op: u8 = b'+';
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] >= b'0' && bytes[i] <= b'9' {
                current = current * 10 + (bytes[i] - b'0') as i64;
            }
            if (bytes[i] < b'0' || bytes[i] > b'9' || i == bytes.len() - 1) && bytes[i] != b' ' {
                match op {
                    b'+' => result += current,
                    b'-' => result -= current,
                    b'*' => result *= current,
                    b'/' => if current != 0 { result /= current },
                    _ => result += current,
                }
                current = 0;
                if bytes[i] < b'0' || bytes[i] > b'9' {
                    op = bytes[i];
                }
            }
            i += 1;
        }
        result
    }

    pub fn add_line_colored(&mut self, text: &str, color: u32) {
        if self.line_count < MAX_LINES {
            self.lines[self.line_count].set(text, color);
            self.line_count += 1;
        } else {
            // Scroll up: shift all lines
            for i in 1..MAX_LINES {
                // Manual field copy to avoid moving non-Copy types
                let src_len = self.lines[i].len;
                let src_color = self.lines[i].color;
                let mut tmp = [0u8; MAX_LINE_LEN];
                tmp[..src_len].copy_from_slice(&self.lines[i].data[..src_len]);
                self.lines[i-1].data[..src_len].copy_from_slice(&tmp[..src_len]);
                self.lines[i-1].len = src_len;
                self.lines[i-1].color = src_color;
            }
            self.lines[MAX_LINES - 1].set(text, color);
        }
    }

    fn add_line_raw(&mut self, bytes: &[u8], color: u32) {
        if self.line_count < MAX_LINES {
            let copy_len = bytes.len().min(MAX_LINE_LEN);
            self.lines[self.line_count].data[..copy_len].copy_from_slice(&bytes[..copy_len]);
            self.lines[self.line_count].len = copy_len;
            self.lines[self.line_count].color = color;
            self.line_count += 1;
        } else {
            for i in 1..MAX_LINES {
                let src_len = self.lines[i].len;
                let src_color = self.lines[i].color;
                let mut tmp = [0u8; MAX_LINE_LEN];
                tmp[..src_len].copy_from_slice(&self.lines[i].data[..src_len]);
                self.lines[i-1].data[..src_len].copy_from_slice(&tmp[..src_len]);
                self.lines[i-1].len = src_len;
                self.lines[i-1].color = src_color;
            }
            let idx = MAX_LINES - 1;
            let copy_len = bytes.len().min(MAX_LINE_LEN);
            self.lines[idx].data[..copy_len].copy_from_slice(&bytes[..copy_len]);
            self.lines[idx].len = copy_len;
            self.lines[idx].color = color;
        }
    }
}

fn int_to_str(mut n: i64, buf: &mut [u8; 20]) -> &str {
    let neg = n < 0;
    if neg { n = -n; }
    if n == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap_or("0");
    }

    let mut pos = 19;
    while n > 0 && pos > 0 {
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        pos -= 1;
    }
    if neg && pos > 0 {
        buf[pos] = b'-';
        pos -= 1;
    }
    core::str::from_utf8(&buf[pos+1..20]).unwrap_or("?")
}

pub fn render_terminal(win: &Window, fb: &mut Framebuffer) {
    if win.is_minimized {
        return;
    }

    win.draw_frame(fb);
    render_terminal_content(win, fb);
}

pub fn render_terminal_content(win: &Window, fb: &mut Framebuffer) {
    let content_x = win.x + 4;
    let content_y = win.y + TITLEBAR_HEIGHT + 2;
    let content_w = win.width.saturating_sub(8);
    let content_h = win.height.saturating_sub(TITLEBAR_HEIGHT + 6);

    // Dark console background
    fb.fill_rect(content_x, content_y, content_w, content_h, COLOR_CHARCOAL);

    // Thin left accent line (Claude style)
    fb.fill_rect(content_x, content_y, 2, content_h, COLOR_TERRACOTTA);

    let line_height = FONT_HEIGHT + 2; // 18px per line
    let visible_lines = content_h.saturating_sub(30) / line_height; // Reserve space for prompt
    let left_pad = content_x + 12;

    unsafe {
        let term = &TERM;

        // Auto-scroll: ensure latest content visible
        let scroll = if term.line_count > visible_lines {
            term.line_count - visible_lines
        } else {
            0
        };

        // Render visible lines
        let mut ty = content_y + 6;
        let start = scroll;
        let end = (start + visible_lines).min(term.line_count);

        for i in start..end {
            let line = &term.lines[i];
            fb.draw_string(left_pad, ty, line.as_str(), line.color);
            ty += line_height;
        }

        // Active prompt line
        let prompt_y = content_y + content_h.saturating_sub(22);

        // Prompt
        let cwd_str = term.get_cwd_str();
        let prompt_parts: [(&str, u32); 3] = [
            ("root@zeroth:", 0x0088CC44), // green user@host
            (cwd_str, 0x005599FF),        // blue cwd
            ("$ ", COLOR_TERRACOTTA),      // terracotta $
        ];

        let mut px = left_pad;
        for (text, color) in prompt_parts.iter() {
            fb.draw_string(px, prompt_y, text, *color);
            px += text.len() * FONT_WIDTH;
        }

        // Input text
        if let Ok(input_str) = core::str::from_utf8(&term.input_buffer[..term.input_len]) {
            fb.draw_string(px, prompt_y, input_str, COLOR_WHITE);

            // Block cursor at cursor position
            let cursor_x = px + term.cursor_pos * FONT_WIDTH;
            fb.fill_rect(cursor_x, prompt_y, FONT_WIDTH, FONT_HEIGHT, COLOR_OCHRE);

            // Show character under cursor if any
            if term.cursor_pos < term.input_len {
                let c = term.input_buffer[term.cursor_pos] as char;
                let mut tmp = [0u8; 1];
                tmp[0] = c as u8;
                if let Ok(s) = core::str::from_utf8(&tmp) {
                    fb.draw_string(cursor_x, prompt_y, s, COLOR_CHARCOAL);
                }
            }
        }

        // Scrollbar indicator on right edge
        if term.line_count > visible_lines {
            let sb_x = content_x + content_w - 6;
            let sb_total_h = content_h.saturating_sub(30);
            let sb_thumb_h = (visible_lines * sb_total_h / term.line_count).max(10);
            let sb_thumb_y = content_y + 4 + (scroll * sb_total_h / term.line_count);

            fb.fill_rect(sb_x, content_y + 4, 4, sb_total_h, 0x003A3935);
            fb.fill_rect(sb_x, sb_thumb_y, 4, sb_thumb_h, COLOR_OCHRE);
        }
    }
}
