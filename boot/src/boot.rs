#![no_std]
#![no_main]
#![allow(dead_code)]

use core::arch::global_asm;

mod ccu;
mod dram;
mod elf;
mod mmio;
mod panic;
mod time;
mod uart;
mod zmodem;

global_asm!(include_str!("boot.S"));

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _main() -> ! {
    unsafe { uart::uart_init() };

    uart::printf!("Bootloader is running\r\n");

    unsafe { ccu::init_clocks() };
    unsafe { dram::init_dram() };

    let zmodem = zmodem::ZModem::new(crate::uart::uart_read, crate::uart::uart_write);
    let mut buffer =
        unsafe { core::slice::from_raw_parts_mut(dram::dram_base(), 1024 * 1024 * 32) };
    let file_size = zmodem.recv_file(&mut buffer);

    uart::printf!("Received file of size %d\r\n", file_size as u64);

    unsafe { elf::execute(buffer.as_mut_ptr()) }
}
