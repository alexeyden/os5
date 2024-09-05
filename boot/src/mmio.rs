pub unsafe fn write32(addr: u64, v: u32) {
    let p = addr as *mut u32;
    core::ptr::write_volatile(p, v);
}

pub unsafe fn read32(addr: u64) -> u32 {
    let p = addr as *const u32;
    core::ptr::read_volatile(p)
}

#[derive(Clone, Copy)]
#[must_use]
pub struct Reg32 {
    p: *mut u32,
    v: u32,
}

impl Reg32 {
    pub unsafe fn read(addr: u64) -> Self {
        let p = addr as *mut u32;
        let v = core::ptr::read_volatile(p);
        Self { p, v }
    }

    pub unsafe fn set_field<const SHIFT: usize, const LEN: usize>(mut self, v: u32) -> Self {
        let mask = !((1u32 << LEN).wrapping_sub(1) << SHIFT);
        self.v &= mask;
        self.v |= v << SHIFT;
        self
    }

    pub unsafe fn is_bit_set<const SHIFT: usize>(&self) -> bool {
        (self.v & (1 << SHIFT)) != 0
    }

    pub unsafe fn field<const SHIFT: usize, const LEN: usize>(&self) -> u32 {
        let mask = (1u32 << LEN).wrapping_sub(1);
        (self.v >> SHIFT) & mask
    }

    pub unsafe fn write(self) {
        core::ptr::write_volatile(self.p, self.v)
    }
}
