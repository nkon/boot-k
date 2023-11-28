//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;

use rp2040_hal::{
    clocks::{init_clocks_and_plls, Clock},
    fugit::RateExtU32, // time calculation library
    gpio::Pins,
    pac,
    sio::Sio,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
};

#[link_section = ".image_header"]
#[used]
pub static IMAGE_HEADER: [u8; 256] = [0u8; 256];

#[entry]
fn main() -> ! {
    info!("App blinky start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Set up UART on GP0 and GP1 (Pico pins 1 and 2)
    let uart_pins = (pins.gpio0.into_function(), pins.gpio1.into_function());
    // Need to perform clock init before using UART or it will freeze.
    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    uart.write_full_blocking(b"app-blinky stated...\r\n");

    #[cfg(debug_assertions)]
    uart.write_full_blocking(b"app-blinky debug build\r\n");

    #[cfg(not(debug_assertions))]
    uart.write_full_blocking(b"app-blinky release build\r\n");

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead. If you have
    // a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here.
    let mut led_pin = pins.gpio25.into_push_pull_output();

    loop {
        uart.write_full_blocking(b"app-blinky on!\r\n");
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        uart.write_full_blocking(b"app-blinky off!\r\n");
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}

