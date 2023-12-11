//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use core::arch::asm;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
// use embedded_hal::digital::v2::OutputPin;
use blxlib::{crc32, image_header};
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

#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; image_header::HEADER_LENGTH as usize] =
    rp2040_boot2::BOOT_LOADER_W25Q080;

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}

#[entry]
fn main() -> ! {
    info!("Program start");
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

    uart.write_full_blocking(b"bootloader stated...\r\n");

    #[cfg(debug_assertions)]
    uart.write_full_blocking(b"bootloader debug build\r\n");

    #[cfg(not(debug_assertions))]
    uart.write_full_blocking(b"bootloader release build\r\n");

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead. If you have
    // a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here.
    let mut _led_pin = pins.gpio25.into_push_pull_output();

    let ih = image_header::load_from_addr(0x1002_0000);
    info!("header_magic: {:04x}", ih.header_magic);
    info!("header_length: {}", ih.header_length);
    info!("hv: {}.{}", ih.hv_major, ih.hv_minor);
    info!(
        "iv: {}.{}.{}-{:08x}",
        ih.iv_major, ih.iv_minor, ih.iv_patch, ih.iv_build
    );
    info!("image_length: {:04x}", ih.image_length);
    info!("payload_crc: {:04x}", ih.payload_crc);
    info!("crc32: {:04x}", ih.crc32);

    // validate header
    if !ih.is_correct_magic() {
        error!("header=magic is not correct: {:04x}", ih.header_magic);
        halt();
    } else {
        info!("header_magic is correct: {:04x}", ih.header_magic)
    }
    if ih.header_length != image_header::HEADER_LENGTH {
        error!("header_length is not correct: {:04x}", ih.header_length);
        halt();
    } else {
        info!("header_length is correct: {:04x}", ih.header_length)
    }
    if !ih.is_correct_crc() {
        error!("crc32 is not correct: {:04x}", ih.crc32);
        halt();
    } else {
        info!("crc32 is correct: {:04x}", ih.crc32)
    }
    let slice = core::ptr::slice_from_raw_parts(
        (0x1002_0000 + image_header::HEADER_LENGTH as usize) as *const u8,
        ih.image_length as usize,
    );
    let payload_crc = crc32::crc32(unsafe { &*slice });
    if ih.payload_crc != payload_crc {
        error!("payload_crc is not correct: {:04x}", ih.payload_crc);
        halt();
    } else {
        info!("payload_crc is correct: {:04x}", ih.payload_crc)
    }

    uart.write_full_blocking(b"bootloader: app header validation pass\r\n");
    uart.write_full_blocking(b"bootloader: boot application!!!\r\n");

    delay.delay_ms(500);

    unsafe {
        asm!(
            "ldr r0, =0x10020100",
            "ldr r1, =0xe000ed08",
            "str r0, [r1]",
            "ldmia r0, {{r0, r1}}",
            "msr msp, r0",
            "bx r1",
        );
    };

    loop {
        cortex_m::asm::wfi();
    }
}
