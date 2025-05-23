#![no_std]
#![no_main]

const UART0_BASE: u64 = 0x02500000;
const UART_USR: u64 = 0x7c;
const UART_THR: u64 = 0x00;

mod panic;

fn uart_write(b: u8) {
    unsafe {
        while core::ptr::read_volatile((UART0_BASE + UART_USR) as *mut u32) & (1 << 1) == 0 {}

        core::ptr::write_volatile((UART0_BASE + UART_THR) as *mut u32, b as u32);
    }
}

fn puts(s: &[u8]) {
    for &b in s {
        uart_write(b);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    puts(b"hello from kernel\r\n");
    loop {}
}
