# Updated YRD0150 E-Paper Display + Button Wiring - Raspberry Pi Pico Pinout

## Complete System Pinout

```
    RASPBERRY PI PICO                    YRD0150 DISPLAY
    +-------------------+                 +-------------+
    |                   |                 |             |
    |   3.3V (Pin 36)   |-----------------| VCC   (1)   |
    |                   |                 |             |
    |   GND  (Pin 38)   |-----------------| GND   (2)   |
    |                   |                 |             |
    |   GP19 (Pin 25)   |-----------------| MOSI  (3)   |
    |                   |                 |             |
    |   GP18 (Pin 24)   |-----------------| SCK   (4)   |
    |                   |                 |             |
    |   GP17 (Pin 22)   |-----------------| CS    (5)   |
    |                   |                 |             |
    |   GP16 (Pin 21)   |-----------------| DC    (6)   |
    |                   |                 |             |
    |   GP15 (Pin 20)   |-----------------| RST   (7)   |
    |                   |                 |             |
    |   GP14 (Pin 19)   |-----------------| BUSY  (8)   |
    |                   |                 |             |
    |   GP12 (Pin 16)   |----+            +-------------+
    |                   |    |
    |   GP13 (Pin 17)   |----+----[ BUTTON A ]----+
    |                   |    |                    |
    |   GND  (Pin 38)   |----+--------------------+---- GND
    |                   |    |                    |
    |   GND  (Pin 13)   |----+----[ BUTTON B ]----+
    |                   |
    +-------------------+
```

## Quick Reference Card

```
╔══════════════╦════════════════╦══════════════╦══════════╦══════════╗
║ Component    ║ Label         ║ Function     ║ Pico GPIO║ Pico Pin ║
╠══════════════╬════════════════╬══════════════╬══════════╬══════════╣
║ Display      ║ VCC           ║ Power 3.3V   ║ 3.3V     ║    36    ║
║ Display      ║ GND           ║ Ground       ║ GND      ║    38    ║
║ Display      ║ DIN (MOSI)    ║ SPI Data     ║ GP19     ║    25    ║
║ Display      ║ CLK (SCK)     ║ SPI Clock    ║ GP18     ║    24    ║
║ Display      ║ CS            ║ Chip Select  ║ GP17     ║    22    ║
║ Display      ║ DC            ║ Data/Command ║ GP16     ║    21    ║
║ Display      ║ RST           ║ Reset        ║ GP15     ║    20    ║
║ Display      ║ BUSY          ║ Busy Status  ║ GP14     ║    19    ║
║ Button A     ║ -             ║ Menu Select  ║ GP12     ║    16    ║
║ Button B     ║ -             ║ Navigate/Back║ GP13     ║    17    ║
║ Common GND   ║ -             ║ Ground       ║ GND      ║ 13,18,38 ║
╚══════════════╩════════════════╩══════════════╩══════════╩══════════╝
```

## Button Wiring Notes

- **No external resistors needed** - Pico has internal pull-ups enabled in software
- **Buttons connect directly to GND** - When pressed, pin reads LOW
- **Debouncing** - Handled in software with delay