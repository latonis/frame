use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Size, OriginDimensions},
    pixelcolor::BinaryColor,
    Pixel,
};

pub struct EpdFramebuffer {
    pub buffer: [u8; 5000],
}

impl EpdFramebuffer {
    pub fn new() -> Self {
        Self { buffer: [0; 5000] }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}

impl Default for EpdFramebuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl OriginDimensions for EpdFramebuffer {
    fn size(&self) -> Size {
        Size::new(200, 200)
    }
}

impl DrawTarget for EpdFramebuffer {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0 && coord.x < 200 && coord.y >= 0 && coord.y < 200 {
                let idx = (coord.y as usize * 200 + coord.x as usize) / 8;
                let bit = 7 - ((coord.y as usize * 200 + coord.x as usize) % 8);
                if color == BinaryColor::On {
                    self.buffer[idx] |= 1 << bit;
                } else {
                    self.buffer[idx] &= !(1 << bit);
                }
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: BinaryColor) -> Result<(), Self::Error> {
        let byte_value = if color == BinaryColor::On { 0xFF } else { 0x00 };
        for byte in self.buffer.iter_mut() {
            *byte = byte_value;
        }
        Ok(())
    }
}
