//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use core::arch::asm;
use core::fmt::Write;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
// use embedded_hal::digital::v2::OutputPin;
use blxlib::{
    crc32,
    image_header::{self, ImageHeader},
};
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
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_RAM_MEMCPY;
// pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

fn ih_print(ih: &ImageHeader) {
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
}

fn ih_validate(ih: &ImageHeader) -> bool {
    // validate header
    if !ih.is_correct_magic() {
        error!("header=magic is not correct: {:04x}", ih.header_magic);
        return false;
    }
    if ih.header_length != image_header::HEADER_LENGTH {
        error!("header_length is not correct: {:04x}", ih.header_length);
        return false;
    }
    if !ih.is_correct_crc() {
        error!("crc32 is not correct: {:04x}", ih.crc32);
        return false;
    }
    let slice = core::ptr::slice_from_raw_parts(
        (0x1002_0000 + image_header::HEADER_LENGTH as usize) as *const u8,
        ih.image_length as usize,
    );
    let payload_crc = crc32::crc32(unsafe { &*slice });
    if ih.payload_crc != payload_crc {
        error!("payload_crc is not correct: {:04x}", ih.payload_crc);
        return false;
    }
    true
}

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}

#[entry]
fn main() -> ! {
    info!("MSP={:08x}", cortex_m::register::msp::read());
    info!("PC={:08x}", cortex_m::register::pc::read());
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
    let mut uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    writeln!(uart, "MSP={:08x}\r", cortex_m::register::msp::read()).unwrap();
    writeln!(uart, "PC={:08x}\r", cortex_m::register::pc::read()).unwrap();
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
    ih_print(&ih);
    if !ih_validate(&ih) {
        uart.write_full_blocking(b"bootloader: FAIL: IMAGE VALIDATION ***\r\n");
        // halt();
    }

    uart.write_full_blocking(b"bootloader: app header validation pass\r\n");
    uart.write_full_blocking(b"bootloader: boot application!!!\r\n");

    delay.delay_ms(500);

    // ldr r3, =XIP_SSI_BASE                   ; XIP_SSI_BASE             0x18000000

    // // Disable SSI to allow further config
    // mov r1, #0
    // str r1, [r3, #SSI_SSIENR_OFFSET]        ; SSI_SSIENR_OFFSET        0x00000008

    // // Set baud rate
    // mov r1, #PICO_FLASH_SPI_CLKDIV          ; PICO_FLASH_SPI_CLKDIV    4
    // str r1, [r3, #SSI_BAUDR_OFFSET]         ; SSI_BAUDR_OFFSET         0x00000014

    // ldr r1, =(CTRLR0_XIP)      ; CTRLR0_XIP  (0x0 << 21) | (31  << 16) | (0x3 << 8)
    // 0b0000_0000_0001_1111_0000_0011_0000_0000 = 0x001f0300
    // str r1, [r3, #SSI_CTRLR0_OFFSET]        ; SSI_CTRLR0_OFFSET        0x00000000

    // ldr r1, =(SPI_CTRLR0_XIP)  ; SPI_CTRLR0_XIP  (CMD_READ << 24) | (2 << 8) | (ADDR_L << 2) | (0x0 << 0)
    // 0b0000_0011_0000_0000_0000_0010_0001_1000 = 0x03000218

    // ldr r0, =(XIP_SSI_BASE + SSI_SPI_CTRLR0_OFFSET); SSI_SPI_CTRLR0_OFFSET    0x000000f4
    // str r1, [r0]

    // // NDF=0 (single 32b read)
    // mov r1, #0x0
    // str r1, [r3, #SSI_CTRLR1_OFFSET]        ; SSI_CTRLR1_OFFSET        0x00000004

    // // Re-enable SSI
    // mov r1, #1
    // str r1, [r3, #SSI_SSIENR_OFFSET]

    unsafe {
        asm!(
            "ldr r3, =0x18000000",
            "movs r1, #0",
            "str r1, [r3, #0x00000008]",
            "movs r1, #4",
            "str r1, [r3, #0x00000014]",
            "ldr r1, =0x001f0300",
            "str r1, [r3, #0x00000000]",
            "ldr r1, =0x03000218",
            "ldr r0, =0x180000f4",
            "str r1, [r0]",
            "movs r1, #0x0",
            "str r1, [r3, #0x00000004]",
            "movs r1, #1",
            "str r1, [r3, #0x00000008]",
        );
    };

    // exec => 0x10020100
    // stack pointer => VTOR[0] (VTOR=0xe000ed08)
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

    halt();
}
