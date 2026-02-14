use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::{InputPin, OutputPin};

pub struct Yrd0150Display<SPI, CS, DC, RST, BUSY, DELAY> {
    spi: SPI,
    cs: CS,
    dc: DC,
    rst: RST,
    busy: BUSY,
    pub delay: DELAY,
}

impl<SPI, CS, DC, RST, BUSY, DELAY> Yrd0150Display<SPI, CS, DC, RST, BUSY, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    DC: OutputPin,
    RST: OutputPin,
    BUSY: InputPin,
    DELAY: DelayMs<u32>,
{
    pub fn new(spi: SPI, cs: CS, dc: DC, rst: RST, busy: BUSY, delay: DELAY) -> Self {
        Self {
            spi,
            cs,
            dc,
            rst,
            busy,
            delay,
        }
    }

    pub fn wait_until_idle(&mut self) {
        while self.busy.is_high().unwrap_or(true) {}
    }

    pub fn send_command(&mut self, command: u8) {
        let _ = self.cs.set_low();
        let _ = self.dc.set_low();  // Command mode
        let _ = self.spi.write(&[command]);
        let _ = self.cs.set_high();
    }

    pub fn send_data(&mut self, data: u8) {
        let _ = self.cs.set_low();
        let _ = self.dc.set_high();  // Data mode
        let _ = self.spi.write(&[data]);
        let _ = self.cs.set_high();
    }

    pub fn send_data_block(&mut self, data: &[u8]) {
        let _ = self.cs.set_low();
        let _ = self.dc.set_high();  // Data mode
        let _ = self.spi.write(data);
        let _ = self.cs.set_high();
    }

    pub fn reset(&mut self) {
        let _ = self.rst.set_low();
        self.delay.delay_ms(10);
        let _ = self.rst.set_high();
        self.delay.delay_ms(10);
        self.wait_until_idle();
    }

    pub fn init(&mut self) {
        self.reset();

        // Booster soft start (0x0C) - from datasheet page 17
        self.send_command(0x0C);
        self.send_data(0x8B);  // Phase1
        self.send_data(0x9C);  // Phase2
        self.send_data(0x96);  // Phase3
        self.send_data(0x0F);  // Duration

        // Driver output control (0x01) - from datasheet page 15
        self.send_command(0x01);
        self.send_data(0xC7);  // Gate 0-199 (0xC7 = 199)
        self.send_data(0x00);  //
        self.send_data(0x00);  // No scan divider

        // Data entry mode (0x11) - from datasheet page 16
        self.send_command(0x11);
        self.send_data(0x03);  // Y increment, X increment

        // Set RAM X start/end (0x44) - from datasheet page 22
        self.send_command(0x44);
        self.send_data(0x00);  // X start: 0
        self.send_data(0x18);  // X end: 24 (200/8 - 1 = 24)

        // Set RAM Y start/end (0x45) - from datasheet page 22
        self.send_command(0x45);
        self.send_data(0xC7);  // Y start low byte: 199
        self.send_data(0x00);  // Y start high byte: 0
        self.send_data(0x00);  // Y end low byte: 0
        self.send_data(0x00);  // Y end high byte: 0

        // Border waveform (0x3C) - from datasheet page 22
        self.send_command(0x3C);
        self.send_data(0xC0);  // VBD as HIZ

        // Temperature sensor control (0x18) - from datasheet page 18
        self.send_command(0x18);
        self.send_data(0x80);  // Use internal temperature sensor
    }

    pub fn set_window(&mut self, x_start: u8, x_end: u8, y_start: u16, y_end: u16) {
        // Set RAM X start/end (0x44)
        self.send_command(0x44);
        self.send_data(x_start);
        self.send_data(x_end);

        // Set RAM Y start/end (0x45)
        self.send_command(0x45);
        self.send_data((y_start & 0xFF) as u8);
        self.send_data(((y_start >> 8) & 0xFF) as u8);
        self.send_data((y_end & 0xFF) as u8);
        self.send_data(((y_end >> 8) & 0xFF) as u8);
    }

    pub fn set_cursor(&mut self, x: u8, y: u16) {
        // Set RAM X counter (0x4E)
        self.send_command(0x4E);
        self.send_data(x);

        // Set RAM Y counter (0x4F)
        self.send_command(0x4F);
        self.send_data((y & 0xFF) as u8);
        self.send_data(((y >> 8) & 0xFF) as u8);
    }

    pub fn clear(&mut self, color: u8) {
        // Set window to full screen
        self.set_window(0, 24, 0, 199);
        self.set_cursor(0, 0);

        // Write RAM (0x24)
        self.send_command(0x24);

        // Fill entire RAM (200*200/8 = 5000 bytes)
        for _ in 0..5000 {
            self.send_data(color);
        }

        self.wait_until_idle();
    }

    pub fn write_framebuffer(&mut self, buffer: &[u8]) {
        // Set window to full screen
        self.set_window(0, 24, 0, 199);
        self.set_cursor(0, 0);

        // Write RAM (0x24)
        self.send_command(0x24);
        self.send_data_block(buffer);
        self.wait_until_idle();
    }

    pub fn update(&mut self) {
        // Display update control 2 (0x22) - from datasheet page 19
        self.send_command(0x22);
        self.send_data(0xFF);  // Enable all stages

        // Master activation (0x20) - from datasheet page 18
        self.send_command(0x20);

        self.wait_until_idle();
    }

    pub fn sleep(&mut self) {
        // Deep sleep mode (0x10) - from datasheet page 15
        self.send_command(0x10);
        self.send_data(0x01);  // Deep sleep mode 1
        self.delay.delay_ms(100);
    }
}
