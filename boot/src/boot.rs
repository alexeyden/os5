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
    unsafe { uart::uart_init() };

    uart::printf!("Hello, world!\r\n");
    uart::printf!("printf formatting test: %s, %d, 0x%x\r\n", "string", -41i64, 0xd00dfeedu64);

    loop {}
}
