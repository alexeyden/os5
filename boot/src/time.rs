use core::arch::asm;

unsafe fn timer_csr() -> u64 {
    let mut timer = core::mem::MaybeUninit::<u64>::uninit();

    unsafe { asm!("csrr {timer}, time", timer = out(reg) * timer.as_mut_ptr()) };

    unsafe { timer.assume_init() }
}

pub fn udelay(us: u64) {
    let mut t1 = unsafe { timer_csr() };
    let t2 = t1 + us * 24;

    while t2 >= t1 {
        t1 = unsafe { timer_csr() };
    }
}
