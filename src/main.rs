#![no_std]
#![no_main]

use core::fmt::Write as FmtWrite;

use cortex_m_rt::entry;
use panic_halt as _;

use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::gpio::FunctionSpi;
use rp_pico::hal::Clock;

use embedded_hal::spi::MODE_0;
use fugit::RateExtU32;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    Drawable,
    mono_font::{ascii::FONT_9X15_BOLD, MonoTextStyle},
    pixelcolor::BinaryColor,
    text::Text,
};
use shared_bus::BusManagerSimple;

mod display;
mod graphics;

mod menu;
mod storage;

use display::Yrd0150Display;
use menu::Menu;
use heapless::String;
use storage::{
    Controller,
    FatError,
    FixedTimeSource,
    SdMmcError as SdError,
    SdMmcSpi,
    load_bmp,
    SpiFullDuplex,
    VolumeIdx,
};

enum AppState {
    Menu,
    ShowingImage,
}

fn sd_status_message(code: u8) -> &'static str {
    match code {
        3 => "FAT MOUNT ERROR",
        4 => "CARD NOT FOUND",
        5 => "CRC SETUP ERROR",
        6 => "SPI ERROR",
        7 => "CMD TIMEOUT",
        8 => "ACMD TIMEOUT",
        9..=13 => "SD INIT ERROR",
        14 => "SD READ ERROR",
        15 => "FAT FORMAT ERROR",
        16 => "NO VOLUME 0",
        17 => "FAT MOUNT ERROR",
        _ => "SD UNKNOWN ERROR",
    }
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::watchdog::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    ).ok().unwrap();

    let sio = hal::sio::Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();
    let system_frequency = clocks.system_clock.freq().to_Hz();
    let delay = cortex_m::delay::Delay::new(core.SYST, system_frequency);

    let button_next = pins.gpio12.into_pull_up_input();    // Next in menu
    let button_select = pins.gpio13.into_pull_up_input();  // Select/Back

    let spi_mosi = pins.gpio19.into_function::<FunctionSpi>();
    let spi_sck = pins.gpio18.into_function::<FunctionSpi>();
    let spi_miso = pins.gpio20.into_function::<FunctionSpi>();

    let spi: hal::spi::Spi<_, _, _, 8> = hal::spi::Spi::new(pac.SPI0, (spi_mosi, spi_miso, spi_sck));
    // 400 kHz is within the SD card's initialization-speed requirement and
    // is also safe for the shared e-ink/SD bus.
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        400_000u32.Hz(),
        MODE_0,
    );
    let bus = BusManagerSimple::new(spi);
    let display_spi = bus.acquire_spi();
    let sd_spi = bus.acquire_spi();

    let display_cs = pins.gpio17.into_push_pull_output();
    let dc = pins.gpio16.into_push_pull_output();
    let rst = pins.gpio15.into_push_pull_output();
    let busy_input = pins.gpio14.into_pull_up_input();

    let mut display = Yrd0150Display::new(display_spi, display_cs, dc, rst, busy_input, delay);
    display.init();

    let mut menu = Menu::new();
    let mut app_state = AppState::Menu;
    let mut last_next = true;
    let mut last_select = true;

    display.write_framebuffer(menu.draw_menu());
    display.update();

    // GPIO 10 is the SD card CS line. The SD card shares SPI0 with the
    // display, but has its own chip-select signal.
    let sd_cs = pins.gpio10.into_push_pull_output();
    let sd_device = SdMmcSpi::new(SpiFullDuplex::new(sd_spi), sd_cs);
    let mut fat_controller = Controller::new(sd_device, FixedTimeSource);
    let init_error_code = match fat_controller.device().init() {
        Ok(()) => 0,
        Err(error) => match error {
            SdError::CardNotFound => 4,
            SdError::CantEnableCRC => 5,
            SdError::Transport => 6,
            SdError::TimeoutCommand(_) => 7,
            SdError::TimeoutACommand(_) => 8,
            SdError::Cmd58Error => 9,
            SdError::TimeoutReadBuffer => 10,
            SdError::TimeoutWaitNotBusy => 11,
            SdError::GpioError => 12,
            _ => 13,
        },
    };
    let sd_initialized = init_error_code == 0;
    let (fat_error_code, mut mounted_volume) = if sd_initialized {
        match fat_controller.get_volume(VolumeIdx(0)) {
            Ok(volume) => (0, Some(volume)),
            Err(FatError::DeviceError(_)) => (14, None),
            Err(FatError::FormatError(_)) => (15, None),
            Err(FatError::NoSuchVolume) => (16, None),
            Err(_) => (17, None),
        }
    } else {
        (0, None)
    };
    let fat_mounted = mounted_volume.is_some();

    // Replace the fallback menu with the card's root-level BMP files.
    if let Some(ref mut volume) = mounted_volume {
        menu.clear_names();

        if let Ok(root) = fat_controller.open_root_dir(volume) {
            let _ = fat_controller.iterate_dir(volume, &root, |entry| {
                let mut name = String::<13>::new();
                let _ = write!(&mut name, "{}", entry.name);
                if name.as_str().ends_with(".BMP")
                                    && !name.as_str().starts_with("._")
                                    && !name.as_str().contains('_') {
                    menu.add_name(name.as_str());

                }
            });
            let _ = fat_controller.close_dir(volume, root);
        }
        display.write_framebuffer(menu.draw_menu());
        display.update();
    }

    let mut sd_image = [0u8; 5000];

    if !fat_mounted {
        let status_code = if sd_initialized {
            fat_error_code
        } else {
            init_error_code
        };
        let mut status_framebuffer = graphics::EpdFramebuffer::new();
        let _ = status_framebuffer.clear(BinaryColor::Off);
        let style = MonoTextStyle::new(&FONT_9X15_BOLD, BinaryColor::On);
        let _ = Text::new("SD STATUS", Point::new(45, 70), style)
            .draw(&mut status_framebuffer);
        let _ = Text::new(sd_status_message(status_code), Point::new(20, 105), style)
            .draw(&mut status_framebuffer);
        let _ = Text::new("RECHECK CARD", Point::new(35, 140), style)
            .draw(&mut status_framebuffer);
        display.write_framebuffer(status_framebuffer.buffer());
        display.update();
    }

    if fat_mounted {
        // Solid LED means SD communication and FAT volume mounting succeeded.
        let _ = led_pin.set_high();
    } else {
        // Codes 4-13 identify SD initialization failures. Codes 14-17
        // identify FAT/device mount failures.
        let flashes = if sd_initialized { fat_error_code } else { init_error_code };
        let _ = led_pin.set_low();
        for _ in 0..flashes {
            let _ = led_pin.set_high();
            display.delay.delay_ms(200);
            let _ = led_pin.set_low();
            display.delay.delay_ms(200);
        }
    }

    loop {
        let next_pressed = button_next.is_low().unwrap_or(false);
        let select_pressed = button_select.is_low().unwrap_or(false);

        match app_state {
            AppState::Menu => {
                if next_pressed && !last_next {
                    menu.next_image();
                    display.write_framebuffer(menu.draw_menu());
                    display.update();
                    display.delay.delay_ms(250);
                }

                if select_pressed && !last_select {
                    if let Some(ref mut volume) = mounted_volume {
                        let filename = menu.selected_name();
                        let _ = load_bmp(
                            &mut fat_controller,
                            volume,
                            filename,
                            &mut sd_image,
                        );
                    }
                    let image_data: &[u8; 5000] = &sd_image;

                    display.clear(0xFF);
                    display.update();
                    display.delay.delay_ms(1000);  // 1 second black

                    display.clear(0x00);
                    display.update();
                    display.delay.delay_ms(1000);  // 1 second white

                    display.write_framebuffer(image_data);
                    display.update();

                    app_state = AppState::ShowingImage;
                    display.delay.delay_ms(250);
                }
            }

            AppState::ShowingImage => {
                if select_pressed && !last_select {
                    display.clear(0xFF);
                    display.update();
                    display.delay.delay_ms(1000);

                    display.clear(0x00);
                    display.update();
                    display.delay.delay_ms(1000);

                    display.write_framebuffer(menu.draw_menu());
                    display.update();

                    app_state = AppState::Menu;
                    display.delay.delay_ms(250);
                }
            }
        }

        last_next = next_pressed;
        last_select = select_pressed;
        display.delay.delay_ms(10);
    }
}
