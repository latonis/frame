#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::gpio::FunctionSpi;
use rp_pico::hal::Clock;

use embedded_hal::spi::MODE_0;
use fugit::RateExtU32;
use embedded_hal::digital::v2::{InputPin};

mod display;
mod graphics;
mod images;
mod menu;

use display::Yrd0150Display;
use menu::Menu;

enum AppState {
    Menu,
    ShowingImage,
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

    let mut _led_pin = pins.led.into_push_pull_output();
    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let button_next = pins.gpio12.into_pull_up_input();    // Next in menu
    let button_select = pins.gpio13.into_pull_up_input();  // Select/Back

    let spi_mosi = pins.gpio19.into_function::<FunctionSpi>();
    let spi_sck = pins.gpio18.into_function::<FunctionSpi>();
    let spi_miso = pins.gpio20.into_function::<FunctionSpi>();

    let spi: hal::spi::Spi<_, _, _, 8> = hal::spi::Spi::new(pac.SPI0, (spi_mosi, spi_miso, spi_sck));
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        1_000_000u32.Hz(),
        MODE_0,
    );

    let cs = pins.gpio17.into_push_pull_output();
    let dc = pins.gpio16.into_push_pull_output();
    let rst = pins.gpio15.into_push_pull_output();
    let busy_input = pins.gpio14.into_pull_up_input();

    let mut display = Yrd0150Display::new(spi, cs, dc, rst, busy_input, delay);
    display.init();

    let mut menu = Menu::new();
    let mut app_state = AppState::Menu;
    let mut last_next = true;
    let mut last_select = true;

    display.write_framebuffer(menu.draw_menu());
    display.update();

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
                    let image_data = menu.get_selected_image();

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
