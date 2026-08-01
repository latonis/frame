use core::fmt::Write as FmtWrite;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::{ascii::FONT_6X10, ascii::FONT_9X15_BOLD, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::Primitive,
    primitives::{Line, PrimitiveStyle},
    text::{Alignment, Text},
    Drawable,
};
use heapless::String;

use crate::graphics::EpdFramebuffer;


pub const MAX_MENU_ITEMS: usize = 32;
const ITEMS_PER_PAGE: usize = 4;
pub type MenuName = String<13>;

pub struct Menu {
    selected_index: usize,
    page_start: usize,
    item_count: usize,
    names: [MenuName; MAX_MENU_ITEMS],
    framebuffer: EpdFramebuffer,
}

impl Menu {
    pub fn new() -> Self {
        let menu = Self {
            selected_index: 0,
            page_start: 0,
            item_count: 0,
            names: core::array::from_fn(|_| String::new()),
            framebuffer: EpdFramebuffer::new(),
        };
        menu
    }

    pub fn clear_names(&mut self) {
        self.selected_index = 0;
        self.page_start = 0;
        self.item_count = 0;
    }

    pub fn add_name(&mut self, name: &str) {
        if self.item_count >= MAX_MENU_ITEMS {
            return;
        }

        self.names[self.item_count].clear();
        let _ = self.names[self.item_count].push_str(name);
        self.item_count += 1;
    }



    pub fn draw_menu(&mut self) -> &[u8] {
        self.framebuffer.clear(BinaryColor::Off).unwrap();

        let title_style = MonoTextStyle::new(&FONT_9X15_BOLD, BinaryColor::On);
        let _ = Text::with_alignment(
            "SELECT IMAGE",
            Point::new(100, 18),
            title_style,
            Alignment::Center,
        )
        .draw(&mut self.framebuffer);

        let page_count = (self.item_count + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
        let current_page = if page_count == 0 {
            0
        } else {
            self.page_start / ITEMS_PER_PAGE + 1
        };
        let mut page_label = heapless::String::<16>::new();
        let _ = write!(&mut page_label, "PAGE {}/{}", current_page, page_count);
        let page_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let _ = Text::with_alignment(
            page_label.as_str(),
            Point::new(190, 18),
            page_style,
            Alignment::Right,
        )
        .draw(&mut self.framebuffer);

        let _ = Line::new(Point::new(20, 28), Point::new(180, 28))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.framebuffer);

        let page_end = (self.page_start + ITEMS_PER_PAGE).min(self.item_count);
        for i in self.page_start..page_end {
            let row = i - self.page_start;
            let y = 45 + (row * 14) as i32;
            let name = self.names[i].as_str();

            if i == self.selected_index {
                let cursor_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
                let _ = Text::new(">", Point::new(25, y), cursor_style)
                    .draw(&mut self.framebuffer);
                let _ = Text::new(name, Point::new(40, y), cursor_style)
                    .draw(&mut self.framebuffer);
            } else {
                let item_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
                let _ = Text::new(name, Point::new(40, y), item_style)
                    .draw(&mut self.framebuffer);
            }
        }

        let inst_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let _ = Text::with_alignment(
            "B:Next  A:Show",
            Point::new(100, 180),
            inst_style,
            Alignment::Center,
        )
        .draw(&mut self.framebuffer);

        self.framebuffer.buffer()
    }

    pub fn next_image(&mut self) {
        if self.item_count != 0 {
            self.selected_index = (self.selected_index + 1) % self.item_count;
            if self.selected_index == 0 {
                self.page_start = 0;
            } else if self.selected_index >= self.page_start + ITEMS_PER_PAGE {
                self.page_start += ITEMS_PER_PAGE;
            }
        }
    }

    pub fn selected_name(&self) -> &str {
        self.names[self.selected_index].as_str()
    }


}
