//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use blxlib::{
    crc32,
    image_header::{self, ImageHeader},
};
use core::arch::asm;
use core::fmt::Write;
use cortex_m_rt::entry;
use defmt_rtt as _;
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

fn ih_validate<
    S: rp2040_hal::uart::State,
    D: rp2040_hal::uart::UartDevice,
    P: rp2040_hal::uart::ValidUartPinout<D>,
>(
    ih: &ImageHeader,
    start_address: u32,
    uart: &mut UartPeripheral<S, D, P>,
) -> bool
where
    UartPeripheral<S, D, P>: Write,
{
    // validate header
    if !ih.is_correct_magic() {
        // error!("header_magic is not correct: {:08x}", ih.header_magic);
        writeln!(
            uart,
            "header_magic is not correct: {:08x}\r",
            ih.header_magic
        )
        .unwrap();
        return false;
    }
    if ih.header_length != image_header::HEADER_LENGTH {
        // error!("header_length is not correct: {:08x}", ih.header_length);
        writeln!(
            uart,
            "header_length is not correct: {:08x}\r",
            ih.header_length
        )
        .unwrap();
        return false;
    }
    if !ih.is_correct_crc() {
        // error!("crc32 is not correct: {:08x}", ih.crc32);
        writeln!(uart, "crc32 is not correct: {:08x}\r", ih.crc32).unwrap();
        return false;
    }
    let slice = core::ptr::slice_from_raw_parts(
        (start_address as usize + image_header::HEADER_LENGTH as usize) as *const u8,
        ih.image_length as usize,
    );
    let payload_crc = crc32::crc32(unsafe { &*slice });
    if ih.payload_crc != payload_crc {
        // error!("payload_crc is not correct: {:08x}", ih.payload_crc);
        writeln!(
            uart,
            "payload_crc is not correct: header={:08x} calc={:08x}\r",
            ih.payload_crc, payload_crc
        )
        .unwrap();
        return false;
    }
    true
}

fn halt() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}

fn copy_image(_to_addr: u32, _from_addr: u32, _size: u32) {
    //     ldr r0, =ROM_FN_TABLE
    //     ldrh r0, [r0]
    //     ldr r2, =ROM_TABLE_LOOKUP
    //     ldrh r2, [r2]

    //     // Query the bootrom function pointer
    //     ldr r1, =0x3443 // 'C','4' for _memcpy44
    //     blx r2

    //     //uint8_t *_memcpy44(uint32_t *dest, uint32_t *src, uint32_t n)
    //     mov r3, r0
    //     ldr r0, =DST
    //     ldr r1, =SRC
    //     ldr r2, =LEN
    //     blx r3
    unsafe {
        asm!(
            "ldr r0, =0x00000014",
            "ldrh r0, [r0]",
            "ldr r2, =0x00000018",
            "ldrh r2, [r2]",
            "ldr r1, =0x3443",
            "blx r2",
            "mov r3, r0",
            "ldr r0, =0x10020000",
            "ldr r1, =0x10100000",
            "ldr r2, =0xe0000",
            "blx r3",
        );
    };
}

fn xip_enable() {
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
}

#[entry]
fn main() -> ! {
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
    // uart.write_full_blocking(b"bootloader stated...\r\n");

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

    let pc = cortex_m::register::pc::read();
    writeln!(uart, "PC={:08x}\r", pc).unwrap();

    uart.write_full_blocking(b"bootloader: check base image\r\n");
    let ih = image_header::load_from_addr(image_header::APP_BASE_ADDR);
    ih_print(&ih, &mut uart);

    if !ih_validate(&ih, image_header::APP_BASE_ADDR, &mut uart) {
        uart.write_full_blocking(b"bootloader: FAIL: IMAGE VALIDATION ***\r\n");
        halt();
    }

    uart.write_full_blocking(b"bootloader: check update image\r\n");
    let ih_update = image_header::load_from_addr(image_header::APP_UPDATE_ADDR);
    ih_print(&ih_update, &mut uart);

    if ih_validate(&ih_update, image_header::APP_UPDATE_ADDR, &mut uart) {
        uart.write_full_blocking(b"bootloader: UPDATE IMAGE FOUND ***\r\n");
        copy_image(
            image_header::APP_BASE_ADDR,
            image_header::APP_UPDATE_ADDR,
            image_header::APP_SIZE,
        );
        uart.write_full_blocking(b"bootloader: UPDATE IMAGE -> BASE IMAGE\r\n");
        // let ih = image_header::load_from_addr(image_header::APP_BASE_ADDR);
        // ih_print(&ih, &mut uart);
    }

    uart.write_full_blocking(b"bootloader: app header validation pass\r\n");
    uart.write_full_blocking(b"bootloader: boot application!!!\r\n");

    delay.delay_ms(500);

    xip_enable();

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
