use embedded_graphics::{
    prelude::*,
    mono_font::{MonoTextStyle, ascii::FONT_6X10, ascii::FONT_9X15_BOLD},
    text::{Text, Alignment},
    geometry::Point,
    primitives::{Rectangle, PrimitiveStyle, Line},
    pixelcolor::BinaryColor,
};

use crate::graphics::EpdFramebuffer;
use crate::images::{IMAGES, IMAGE_COUNT};

pub struct Menu {
    selected_index: usize,
    framebuffer: EpdFramebuffer,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            framebuffer: EpdFramebuffer::new(),
        }
    }

    pub fn draw_menu(&mut self) -> &[u8] {
        self.framebuffer.clear(BinaryColor::Off).unwrap();

        let title_style = MonoTextStyle::new(&FONT_9X15_BOLD, BinaryColor::On);
        let _ = Text::with_alignment(
            "SELECT IMAGE",
            Point::new(100, 18),
            title_style,
            Alignment::Center,
        ).draw(&mut self.framebuffer);

        let _ = Line::new(Point::new(20, 28), Point::new(180, 28))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.framebuffer);

        for (i, image) in IMAGES.iter().enumerate() {
            let y = 45 + (i * 14) as i32;

            if i == self.selected_index {
                let _ = Rectangle::new(Point::new(20, y - 8), Size::new(160, 12))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(&mut self.framebuffer);

                let highlight_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);
                let _ = Text::new("►", Point::new(25, y), highlight_style)
                    .draw(&mut self.framebuffer);
                let _ = Text::new(image.name, Point::new(40, y), highlight_style)
                    .draw(&mut self.framebuffer);
            } else {
                let item_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
                let _ = Text::new("○", Point::new(25, y), item_style)
                    .draw(&mut self.framebuffer);
                let _ = Text::new(image.name, Point::new(40, y), item_style)
                    .draw(&mut self.framebuffer);
            }
        }

        let inst_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let _ = Text::with_alignment(
            "B:Next  A:Show",
            Point::new(100, 180),
            inst_style,
            Alignment::Center,
        ).draw(&mut self.framebuffer);

        self.framebuffer.buffer()
    }

    pub fn next_image(&mut self) {
        self.selected_index = (self.selected_index + 1) % IMAGE_COUNT;
    }

    pub fn get_selected_image(&self) -> &'static [u8; 5000] {
        IMAGES[self.selected_index].data
    }
}
