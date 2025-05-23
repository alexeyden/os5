pub unsafe fn write32(addr: u64, v: u32) {
    let p = addr as *mut u32;
    unsafe { core::ptr::write_volatile(p, v) };
}

pub unsafe fn read32(addr: u64) -> u32 {
    let p = addr as *const u32;
    unsafe { core::ptr::read_volatile(p) }
}

#[derive(Clone, Copy)]
#[must_use]
pub struct Reg32 {
    p: *mut u32,
    v: u32,
}

impl Reg32 {
    pub fn zero(addr: u64) -> Self {
        let p = addr as *mut u32;
        Self { p, v: 0 }
    }

    pub unsafe fn read(addr: u64) -> Self {
        let p = addr as *mut u32;
        let v = unsafe { core::ptr::read_volatile(p) };
        Self { p, v }
    }

    pub unsafe fn set_field<const SHIFT: usize, const LEN: usize>(mut self, v: u32) -> Self {
        let mask = !((1u32 << LEN).wrapping_sub(1) << SHIFT);
        self.v &= mask;
        self.v |= v << SHIFT;
        self
    }

    pub unsafe fn and(mut self, v: u32) -> Self {
        self.v &= v;
        self
    }

    pub unsafe fn or(mut self, v: u32) -> Self {
        self.v |= v;
        self
    }

    pub unsafe fn wait_bit<const SHIFT: usize>(mut self, v: bool) {
        while self.is_bit_set::<SHIFT>() != v {
            self.v = unsafe { core::ptr::read_volatile(self.p) };
        }
    }

    pub fn is_bit_set<const SHIFT: usize>(&self) -> bool {
        (self.v & (1 << SHIFT)) != 0
    }

    pub unsafe fn field<const SHIFT: usize, const LEN: usize>(&self) -> u32 {
        let mask = (1u32 << LEN).wrapping_sub(1);
        (self.v >> SHIFT) & mask
    }

    pub unsafe fn write(self) {
        unsafe { core::ptr::write_volatile(self.p, self.v) }
    }
}
