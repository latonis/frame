# frame

`frame` is a small Raspberry Pi Pico project that uses an e-ink display to show photos that bring me joy. The current firmware includes two images compiled into the Pico. An SD card reader is planned for a future version.

## Hardware

- Raspberry Pi Pico
- WeAct Studio 1.54-inch e-ink display
- SPI microSD card module
- Two momentary buttons

## Display wiring

The display is connected using SPI. On this display board, the SPI pins are labeled `SDA` and `SCL`; they are not I²C pins in this setup.

| Display pin | Function | Pico GPIO | Pico physical pin |
|---|---|---:|---:|
| `SDA` | SPI data / MOSI | GPIO 19 | 25 |
| `SCL` | SPI clock / SCK | GPIO 18 | 24 |
| `CS` | Chip select | GPIO 17 | 22 |
| `DC` | Data/command | GPIO 16 | 21 |
| `RST` | Reset | GPIO 15 | 20 |
| `BUSY` | Display busy status | GPIO 14 | 19 |
| `VCC` | Power | 3.3 V | 36 |
| `GND` | Ground | GND | 38, for example |

Use 3.3 V power and logic for the display. Make sure the Pico and display share a ground connection.

## SD card wiring

The SD module uses SPI and shares the display's `MOSI`, `CLK`, and `MISO` lines. It has its own chip-select line so the Pico can select either device independently.

| SD module pin | Function | Pico GPIO | Pico physical pin |
|---|---|---:|---:|
| `3V3` | Power | 3.3 V | 36 |
| `GND` | Ground | GND | 38, for example |
| `CS` | Chip select | GPIO 10 | 14 |
| `MOSI` | SPI data | GPIO 19 | 25 |
| `CLK` | SPI clock | GPIO 18 | 24 |
| `MISO` | SPI data back | GPIO 20 | 26 |

The chip-select lines are:

```text
E-ink display CS -> GPIO 17
SD card CS       -> GPIO 10
```

The firmware now initializes the SD card after initializing the display. The Pico's onboard LED is turned on when SD initialization succeeds and remains off when initialization fails. The SD card and display share SPI0, and the software keeps their chip-select lines separate.

The firmware uses `embedded-sdmmc` to initialize the card and mount volume 0, including FAT32 volumes. It scans the root directory for `.BMP` files and displays up to 32 filenames in pages of four. The selected BMP is loaded when you press Select. The menu is empty if no valid BMP files are found.

## Button wiring

Connect each momentary button between its GPIO pin and GND. The firmware uses the Pico's internal pull-up resistors, so external resistors are not required.

| Button | Pico GPIO | Pico physical pin |
|---|---:|---:|
| Next image | GPIO 12 | 16 |
| Select/back | GPIO 13 | 17 |

## SD card smoke test

Format the card as FAT32, insert it into the module, and power the Pico. Flash the firmware with the build instructions below. The display should still show the normal image menu; the onboard LED indicates SD initialization and FAT volume-mount status:

- Solid LED: SD card initialized and volume 0 mounted successfully
- Four flashes: card was not detected
- Five flashes: card rejected CRC setup
- Six flashes: SPI transport error
- Seven or eight flashes: SD command timeout
- Nine through thirteen flashes: later SD initialization error
- Fourteen flashes: SD read failed while mounting FAT
- Fifteen flashes: FAT format/parser error
- Sixteen flashes: no volume 0 was found
- Seventeen flashes: other FAT mounting error

If initialization fails, check the SD module's 3.3 V power, shared ground, SPI wiring, and GPIO 10 chip select.

## BMP images

The firmware scans the card's root directory for files ending in `.BMP`. Use short 8.3-style names such as:

```text
PHOTO1.BMP
PHOTO2.BMP
PHOTO3.BMP
```

The loader currently supports only:

- 200×200 pixels
- Uncompressed 24-bit BMP
- Standard bottom-up or top-down BMP layout
- Any RGB image converted to black and white using a luminance threshold

For example, export or convert images on your computer as 200×200, 24-bit, uncompressed BMPs and copy them to the root of the SD card. The menu displays the discovered filenames, and selecting one loads that file. Files that cannot be decoded produce a blank image rather than using firmware fallback data.

## Build and flash

Install the required target and UF2 flashing tool once:

```sh
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```

Put the Pico into bootloader mode by holding **BOOTSEL** while plugging it into USB. Then, from the project directory, run:

```sh
cargo run --release
```

The project is configured to build for the Pico's RP2040 (`thumbv6m-none-eabi`) and use `elf2uf2-rs` to flash the resulting firmware. After flashing, the Pico reboots and runs the program automatically.

## Using the device

After startup, the display shows an image-selection menu:

```text
SELECT IMAGE

> PHOTO1.BMP
  PHOTO2.BMP

B:Next  A:Show
```

- Press the **Next** button on GPIO 12 to move through the images.
- Press the **Select/back** button on GPIO 13 to show the selected image.
- Press the **Select/back** button again while viewing an image to return to the menu.

E-ink refreshes can take a few seconds. The display may briefly flash during an update; this is expected.

## Current image support

Images are loaded from the FAT32 SD card as 200×200 BMP files. Copy new files to the card and power-cycle the Pico to refresh the menu.
