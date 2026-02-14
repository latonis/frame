# YRD0150 E-Paper Display Driver Implementation

## Driver Implementation with Datasheet References

This document maps the driver code to the official YRD0150BBS810F0 datasheet specifications.

## 1. Pin Mapping (Datasheet Page 7)

| Driver Parameter | Display Pin | Pin Name | Datasheet Ref |
|-----------------|-------------|----------|---------------|
| `SPI` | 14 | SDA | Section 5, Pin 14 |
| `CS` | 13 | CS# | Section 5, Pin 13, Note 5-1 |
| `DC` | 12 | D/C# | Section 5, Pin 12, Note 5-2 |
| `RST` | 11 | RES# | Section 5, Pin 11, Note 5-3 |
| `BUSY` | 9 | BUSY | Section 5, Pin 9, Note 5-4 |

## 2. SPI Communication Protocol (Datasheet Pages 10-11)

### 4-Wire SPI Timing Reference
```
Section 6.4.2, Figure 6-1: Write procedure in 4-wire SPI mode
Section 6.4.4: Serial Interface Timing Characteristics (Page 14)
```

```rust
// Command mode: D/C# = LOW (Datasheet Table 6-3-1, Page 10)
pub fn send_command(&mut self, command: u8) {
    let _ = self.cs.set_low();      // CS# active low
    let _ = self.dc.set_low();       // Command mode
    let _ = self.spi.write(&[command]); // Data on rising edge of SCL
    let _ = self.cs.set_high();      // CS# inactive high
}

// Data mode: D/C# = HIGH (Datasheet Table 6-3-1, Page 10)
pub fn send_data(&mut self, data: u8) {
    let _ = self.cs.set_low();      // CS# active low
    let _ = self.dc.set_high();      // Data mode
    let _ = self.spi.write(&[data]);  // Data on rising edge of SCL
    let _ = self.cs.set_high();      // CS# inactive high
}
```

## 3. Initialization Sequence (Datasheet Page 25)

### Command Reference Table (Pages 15-22)

```rust
pub fn init(&mut self) {
    self.reset();  // Hardware reset per datasheet Section 10.1
    
    // Booster Soft Start (0x0C) - Datasheet Page 17
    // Controls the driving strength and duration for power stages
    self.send_command(0x0C);
    self.send_data(0x8B);  // Phase 1: Driving strength, Min off time
    self.send_data(0x9C);  // Phase 2: Driving strength, Min off time
    self.send_data(0x96);  // Phase 3: Driving strength, Min off time
    self.send_data(0x0F);  // Duration settings for phases 1-3
    
    // Driver Output Control (0x01) - Datasheet Page 15
    // Sets gate output range (0-199 for 200 rows)
    self.send_command(0x01);
    self.send_data(0xC7);  // Gate 0-199 (0xC7 = 199)
    self.send_data(0x00);  // 
    self.send_data(0x00);  // No scan divider
    
    // Data Entry Mode (0x11) - Datasheet Page 16
    // Sets address increment direction
    self.send_command(0x11);
    self.send_data(0x03);  // Y increment, X increment (AM=1, ID=11)
    
    // Set RAM X Start/End (0x44) - Datasheet Page 22
    // X address range: 0 to 24 (200 pixels / 8 = 25 bytes)
    self.send_command(0x44);
    self.send_data(0x00);  // X start: 0
    self.send_data(0x18);  // X end: 24 (0x18 = 24)
    
    // Set RAM Y Start/End (0x45) - Datasheet Page 22
    // Y address range: 0 to 199 (9-bit values)
    self.send_command(0x45);
    self.send_data(0xC7);  // Y start low byte: 199
    self.send_data(0x00);  // Y start high byte: 0
    self.send_data(0x00);  // Y end low byte: 0
    self.send_data(0x00);  // Y end high byte: 0
    
    // Border Waveform (0x3C) - Datasheet Page 22
    // Controls VBD (border) output
    self.send_command(0x3C);
    self.send_data(0xC0);  // VBD as HiZ (bits 7-6 = 11)
    
    // Temperature Sensor Control (0x18) - Datasheet Page 18
    // Select internal temperature sensor
    self.send_command(0x18);
    self.send_data(0x80);  // Internal sensor (0x80)
}
```

## 4. Display Update Sequence (Datasheet Page 25, Section 10.1)

### Update Flow from Datasheet:
```
1. Send image data to RAM (0x24)
2. Wait for BUSY = Low
3. Send Master Activation (0x20)
4. Wait for BUSY = Low
```

```rust
pub fn write_framebuffer(&mut self, buffer: &[u8]) {
    // Set window to full screen
    self.set_window(0, 24, 0, 199);
    self.set_cursor(0, 0);
    
    // Write RAM (0x24) - Datasheet Page 20
    // Write Black/White RAM
    self.send_command(0x24);
    self.send_data_block(buffer);
    self.wait_until_idle();  // Wait for BUSY low
}

pub fn update(&mut self) {
    // Display Update Control 2 (0x22) - Datasheet Page 19
    // Enable display update sequence stages
    self.send_command(0x22);
    self.send_data(0xFF);  // Enable all stages (Master Activation)
    
    // Master Activation (0x20) - Datasheet Page 18
    // Start display update sequence
    self.send_command(0x20);
    self.wait_until_idle();  // Wait for update complete
}
```

## 5. Power Management (Datasheet Page 15)

### Deep Sleep Mode - Datasheet Page 15, Command 0x10
| Mode | Value | Description |
|------|-------|-------------|
| Normal | 0x00 | Normal operation |
| Deep Sleep 1 | 0x01 | Sleep mode 1 (POR default) |
| Deep Sleep 2 | 0x11 | Sleep mode 2 |

```rust
pub fn sleep(&mut self) {
    // Deep Sleep Mode (0x10) - Datasheet Page 15
    self.send_command(0x10);
    self.send_data(0x01);  // Deep sleep mode 1
    self.delay.delay_ms(100);
    
    // Note: To exit deep sleep, hardware reset is required
}
```

## 6. Memory Layout

### RAM Organization (Datasheet Page 20)
```
Resolution: 200 × 200 pixels
Memory size: 200 × 200 / 8 = 5000 bytes

Pixel mapping:
- Each byte represents 8 horizontal pixels
- MSB = leftmost pixel (bit 7)
- LSB = rightmost pixel (bit 0)
```

```rust
pub struct EpdFramebuffer {
    pub buffer: [u8; 5000],  // 200*200/8 bytes
}

// Pixel manipulation based on SSD1681 datasheet
fn set_pixel(buffer: &mut [u8], x: i32, y: i32, color: BinaryColor) {
    if x >= 0 && x < 200 && y >= 0 && y < 200 {
        let idx = (y as usize * 200 + x as usize) / 8;
        let bit = 7 - ((y as usize * 200 + x as usize) % 8);
        if color == BinaryColor::On {
            buffer[idx] |= 1 << bit;   // Set pixel black
        } else {
            buffer[idx] &= !(1 << bit); // Set pixel white
        }
    }
}
```

## 7. BUSY Pin Behavior (Datasheet Page 8, Note 5-4)

BUSY is high during:
- Outputting display waveform
- Communicating with temperature sensor
- OTP programming

```rust
pub fn wait_until_idle(&mut self) {
    // Wait for BUSY pin to go low (not busy)
    while self.busy.is_high().unwrap_or(true) {}
}
```

## 8. Complete Command Table Reference

| Command | Hex | Datasheet Page | Function |
|---------|-----|----------------|----------|
| Driver Output Control | 0x01 | Page 15 | Set gate output range |
| Deep Sleep Mode | 0x10 | Page 15 | Enter low power mode |
| Data Entry Mode | 0x11 | Page 16 | Set address increment |
| Software Reset | 0x12 | Page 18 | Reset commands and parameters |
| Temperature Sensor Control | 0x18 | Page 18 | Select temp sensor source |
| Master Activation | 0x20 | Page 18 | Start display update |
| Display Update Control 2 | 0x22 | Page 19 | Enable update stages |
| Write RAM (BW) | 0x24 | Page 20 | Write to black/white RAM |
| Border Waveform | 0x3C | Page 22 | Control VBD output |
| Set RAM X Start/End | 0x44 | Page 22 | Set X address window |
| Set RAM Y Start/End | 0x45 | Page 22 | Set Y address window |
| Set RAM X Counter | 0x4E | Page 22 | Set X address position |
| Set RAM Y Counter | 0x4F | Page 22 | Set Y address position |
| Booster Soft Start | 0x0C | Page 17 | Power stage control |

## 9. Typical Operating Sequence (Datasheet Page 25-26)

From Section 10.1 OTP Operation Flow:
```
1. Power ON
2. Hardware Reset (RES# low then high)
3. Wait for BUSY low
4. Software Reset (0x12)
5. Wait for BUSY low
6. Set voltage and load LUT
7. Load image (0x24)
8. Master Activation (0x20)
9. Wait for BUSY low
10. Deep Sleep (0x10)
```

## 10. Electrical Characteristics (Datasheet Page 9)

| Parameter | Value | Condition |
|-----------|-------|-----------|
| VCI | 2.2V - 3.7V | Logic supply |
| Typical power | 10.5 mW | Full refresh @ 3.0V |
| Deep sleep | 0.003 mW | DC/DC off |
| Update time | ~4 sec | @ 25°C |
| Peak current | 25 mA | During update |
