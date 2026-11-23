//! ZeroGUI Window Frame and Compositor

use crate::drivers::framebuffer::*;

pub const TITLEBAR_HEIGHT: usize = 28;

#[derive(Copy, Clone, PartialEq)]
pub enum AppType {
    Browser,
    Terminal,
    Store,
    Calculator,
    Files,
}

pub struct Window {
    pub id: usize,
    pub title: &'static str,
    pub app_type: AppType,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub is_minimized: bool,
    pub is_focused: bool,
}

impl Window {
    pub const fn new(id: usize, title: &'static str, app_type: AppType, x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            id,
            title,
            app_type,
            x,
            y,
            width,
            height,
            is_minimized: false,
            is_focused: true,
        }
    }

    pub fn draw_frame(&self, fb: &mut Framebuffer) {
        if self.is_minimized {
            return;
        }

        let border_color = if self.is_focused { COLOR_TERRACOTTA } else { COLOR_OCHRE };
        let title_bg = if self.is_focused { COLOR_CHARCOAL } else { 0x004A4742 };

        // 1. Shadow
        fb.fill_rect(self.x + 4, self.y + 4, self.width, self.height, 0x00D6D1C5);

        // 2. Main Window Background
        fb.fill_rect(self.x, self.y, self.width, self.height, COLOR_CARD_BG);

        // 3. Titlebar
        fb.fill_rect(self.x, self.y, self.width, TITLEBAR_HEIGHT, title_bg);

        // Title text
        let title_x = self.x + 14;
        let title_y = self.y + 7;
        fb.draw_string(title_x, title_y, self.title, COLOR_WHITE);

        // 4. Close [X] and Minimize [-] buttons
        let btn_y = self.y + 7;
        let close_btn_x = self.x + self.width.saturating_sub(24);
        let min_btn_x = close_btn_x.saturating_sub(20);

        fb.fill_rect(close_btn_x, btn_y, 14, 14, COLOR_TERRACOTTA);
        fb.draw_string(close_btn_x + 3, btn_y, "x", COLOR_WHITE);

        fb.fill_rect(min_btn_x, btn_y, 14, 14, COLOR_OCHRE);
        fb.draw_string(min_btn_x + 4, btn_y, "-", COLOR_CHARCOAL);

        // 5. Border Outline
        fb.draw_rect_outline(self.x, self.y, self.width, self.height, border_color, 2);
    }

    pub fn is_inside_close(&self, mx: usize, my: usize) -> bool {
        let close_x = self.x + self.width.saturating_sub(24);
        let btn_y = self.y + 7;
        mx >= close_x && mx <= close_x + 16 && my >= btn_y && my <= btn_y + 16
    }

    pub fn is_inside_titlebar(&self, mx: usize, my: usize) -> bool {
        mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + TITLEBAR_HEIGHT
    }
}
