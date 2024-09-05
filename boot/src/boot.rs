#![no_std]
#![no_main]
#![allow(dead_code)]

use core::arch::global_asm;

mod mmio;
mod panic;
mod uart;

global_asm!(include_str!("boot.S"));

#[no_mangle]
pub extern "C" fn _main() -> ! {
    use core::fmt::Write;

    unsafe { uart::uart_init() };

    core::write!(&mut uart::UART0, "Hello, world!\r\n").unwrap();
    core::write!(&mut uart::UART0, "Formatting test: {}, {}, 0x{:x}\r\n", "string", -42, 0xd00dfeedu64).unwrap();

    loop {}
}
