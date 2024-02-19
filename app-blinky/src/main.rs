//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use blxlib::image_header::{self, ImageHeader};
use core::{fmt::Write, ptr};
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _; // used by panic-probe
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
pub static IMAGE_HEADER: image_header::ImageHeader = image_header::ImageHeader {
    header_magic: image_header::IMAGE_HEADER_MAGIC,
    header_length: image_header::HEADER_LENGTH,
    hv_major: image_header::HV_MAJOR,
    hv_minor: image_header::HV_MINOR,
    iv_major: 0,
    iv_minor: 1,
    iv_patch: 0,
    iv_build: 0,
    image_length: 0xe_0000,
    signature: [0u8; 128],
    payload_crc: 0,
    padding: [0u8; 100],
    crc32: 0,
};

fn ih_print<
    S: rp2040_hal::uart::State,
    D: rp2040_hal::uart::UartDevice,
    P: rp2040_hal::uart::ValidUartPinout<D>,
>(
    ih: &ImageHeader,
    uart: &mut UartPeripheral<S, D, P>,
) where
    UartPeripheral<S, D, P>: Write,
{
    // info!("header_magic: {:08x}", ih.header_magic);
    // info!("header_length: {}", ih.header_length);
    // info!("hv: {}.{}", ih.hv_major, ih.hv_minor);
    // info!(
    //     "iv: {}.{}.{}-{:08x}",
    //     ih.iv_major, ih.iv_minor, ih.iv_patch, ih.iv_build
    // );
    // info!("image_length: {:08x}", ih.image_length);
    // info!("payload_crc: {:08x}", ih.payload_crc);
    // info!("crc32: {:08x}", ih.crc32);
    writeln!(uart, "header_magic: {:08x}\r", ih.header_magic).unwrap();
    writeln!(uart, "header_length: {}\r", ih.header_length).unwrap();
    writeln!(uart, "hv: {}.{}\r", ih.hv_major, ih.hv_minor).unwrap();
    writeln!(
        uart,
        "iv: {}.{}.{}-{:08x}\r",
        ih.iv_major, ih.iv_minor, ih.iv_patch, ih.iv_build
    )
    .unwrap();
    writeln!(uart, "image_length: {:08x}\r", ih.image_length).unwrap();
    writeln!(uart, "payload_crc: {:08x}\r", ih.payload_crc).unwrap();
    writeln!(uart, "crc32: {:08x}\r", ih.crc32).unwrap();
}


#[entry]
fn main() -> ! {
    info!("app-blinky: defmt-rtt");
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
    let mut uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    writeln!(uart, "MSP={:08x}\r", cortex_m::register::msp::read()).unwrap();
    writeln!(uart, "PC={:08x}\r", cortex_m::register::pc::read()).unwrap();

    uart.write_full_blocking(b"app-blinky started...BBB\r\n");

    #[cfg(debug_assertions)]
    writeln!(&mut uart, "app-blinky debug build\r").unwrap();

    #[cfg(not(debug_assertions))]
    writeln!(&mut uart, "app-blinky release build\r").unwrap();

    let ih = unsafe { ptr::read_volatile(0x1002_0000 as *const ImageHeader) };
    ih_print(&ih, &mut uart);

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead. If you have
    // a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here.
    let mut led_pin = pins.gpio25.into_push_pull_output();

    loop {
        writeln!(uart, "app-blinky on!\r").unwrap();
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        writeln!(uart, "app-blinky off!\r").unwrap();
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}
