use core::panic::PanicInfo;

extern "C" {
    pub fn rust_panic_called_where_shouldnt() -> !;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { rust_panic_called_where_shouldnt(); }
}
