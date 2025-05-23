use core::panic::PanicInfo;

unsafe extern "C" {
    pub unsafe fn rust_panic_called_where_shouldnt() -> !;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { rust_panic_called_where_shouldnt(); }
}

#[macro_export]
macro_rules! boot_panic {
    ($msg:expr $(,$arg:expr)*) => {{
        $crate::uart::printf!(concat!("panic in bootloader: ",$msg), $($arg),*);
        loop {}
    }};
}
