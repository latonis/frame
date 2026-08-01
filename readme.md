# frame

`frame` is a small Raspberry Pi Pico project that uses an e-ink display to show photos that bring me joy. The current firmware includes two images compiled into the Pico. An SD card reader is planned for a future version.

## Hardware

- Raspberry Pi Pico
- WeAct Studio 1.54-inch e-ink display
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

## Button wiring

Connect each momentary button between its GPIO pin and GND. The firmware uses the Pico's internal pull-up resistors, so external resistors are not required.

| Button | Pico GPIO | Pico physical pin |
|---|---:|---:|
| Next image | GPIO 12 | 16 |
| Select/back | GPIO 13 | 17 |

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

► Photo 1
  Photo 2

B:Next  A:Show
```

- Press the **Next** button on GPIO 12 to move through the images.
- Press the **Select/back** button on GPIO 13 to show the selected image.
- Press the **Select/back** button again while viewing an image to return to the menu.

E-ink refreshes can take a few seconds. The display may briefly flash during an update; this is expected.

## Current image support

Images are currently compiled into the firmware rather than read from an SD card. The image list is defined in `src/images/mod.rs`, and the image data is stored in `src/images/image1.rs` and `src/images/image2.rs`. Reflash the Pico after changing an image.
