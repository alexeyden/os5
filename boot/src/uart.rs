use crate::mmio;

const CCU_BASE: u64 = 0x02001000;
const CCU_UART_BGR_REG: u64 = 0x090C;

const UART0_BASE: u64 = 0x02500000;
const UART_LCR: u64 = 0x0c;
const UART_DLL: u64 = 0x00;
const UART_DLH: u64 = 0x04;
const UART_FCR: u64 = 0x08;
const UART_HALT: u64 = 0xa4;
const UART_USR: u64 = 0x7c;
const UART_THR: u64 = 0x00;
const UART_RBR: u64 = 0x00;

const GPIO_BASE: u64 = 0x0200_0000;
const GPIO_PD_CFG2: u64 = 0x98;
const GPIO_PD_DAT: u64 = 0xa0;

const GPIO_PB_CFG1: u64 = 0x0034;
const GPIO_PB_PULL: u64 = 0x0054;

pub unsafe fn uart_init() {
    // Step 1
    mmio::Reg32::read(CCU_BASE + CCU_UART_BGR_REG)
        .set_field::<0, 1>(1) // UART0 gating enable
        .set_field::<16, 1>(1) // UART0 reset deassert
        .write();

    // Step 2
    // Configure pinmux
    mmio::Reg32::read(GPIO_BASE + GPIO_PB_CFG1)
        .set_field::<0, 4>(0b0110) // PB8 = UART0-TX
        .set_field::<4, 4>(0b0110) // PB9 = UART0-RX
        .write();

    mmio::Reg32::read(GPIO_BASE + GPIO_PB_PULL)
        .set_field::<16, 2>(1) // PB8_PULL = Pull_up
        .set_field::<18, 2>(1) // PB9_PULL = Pull_up
        .write();

    // Configure baud rate
    mmio::Reg32::read(UART0_BASE + UART_FCR)
        .set_field::<0, 1>(1) // FIFOE = 1 (enable FIFO)
        .write();

    mmio::Reg32::read(UART0_BASE + UART_HALT)
        .set_field::<0, 1>(1) // HALT_TX = 1
        .write();

    mmio::Reg32::read(UART0_BASE + UART_LCR)
        .set_field::<7, 1>(1) // DLAB = 1 (enable divisor latch register access)
        .write();

    mmio::Reg32::read(UART0_BASE + UART_DLL)
        .set_field::<0, 8>(13) // DLL = 13 (divisor Latch = 13, baud rate = 115200)
        .write();

    mmio::Reg32::read(UART0_BASE + UART_DLH)
        .set_field::<0, 8>(0) // DLH = 0 (divisor Latch = 13, baud rate = 115200)
        .write();

    mmio::Reg32::read(UART0_BASE + UART_LCR)
        .set_field::<7, 1>(0) // DLAB = 0 (disable divisor latch register access)
        .write();

    mmio::Reg32::read(UART0_BASE + UART_HALT)
        .set_field::<0, 1>(0) // HALT_TX = 0
        .write();

    // Step 3
    // Setup mode
    mmio::Reg32::read(UART0_BASE + UART_LCR)
        .set_field::<0, 2>(0b11) // Data Length Select = 8 bits
        .set_field::<2, 1>(0) // 1 stop bit
        .set_field::<3, 1>(0) // partiy disabled
        .set_field::<4, 2>(0) // partiy mode (doesnt really matter because partiy is disabled)
        .set_field::<6, 1>(0) // break control = 0
        .write();

    // Setup FIFO
    mmio::Reg32::read(UART0_BASE + UART_FCR)
        .set_field::<0, 1>(1) // FIFOE = 1 (enable FIFO)
        .set_field::<1, 1>(1) // RFIFOR = 1 (reset rx FIFO)
        .set_field::<2, 1>(1) // XFIFOR = 1 (reset tx FIFO)
        .write();
}

pub unsafe fn uart_write(b: u8) {
    while !mmio::Reg32::read(UART0_BASE + UART_USR).is_bit_set::<1>() {
        // wait for a free space in FIFO (UART_USR[TFNF] = 1)
    }

    mmio::write32(UART0_BASE + UART_THR, b as u32);
}

pub unsafe fn uart_read() -> u8 {
    while !mmio::Reg32::read(UART0_BASE + UART_USR).is_bit_set::<3>() {
        // wait until there is something in FIFO (UART_USR[RFNE] = 1)
    }

    (mmio::read32(UART0_BASE + UART_RBR) & 0xff) as u8
}

pub struct UART0;

impl core::fmt::Write for UART0 {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &b in s.as_bytes() {
            unsafe { uart_write(b) };
        }

        Ok(())
    }
}
