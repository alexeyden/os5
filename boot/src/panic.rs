use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    let _ = core::write!(&mut crate::uart::UART0, "panic: {}", info);
    loop {}
}
