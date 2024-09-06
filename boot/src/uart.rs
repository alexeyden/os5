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

fn print_hex(v: u64) {
    let mut shift = 60usize;
    let mut lz = true;

    loop {
        let digit = ((v >> shift) as u8) & 0xf;

        if digit != 0 || !lz {
            lz = false;

            if digit < 10 {
                unsafe { uart_write(b'0' + digit) };
            } else {
                unsafe { uart_write(b'a' + digit - 10) };
            }
        }

        if shift == 0 {
            break;
        }

        shift -= 4;
    }
}

fn print_dec(mut v: u64) {
    let mut buf: [u8; 20] = [b'0'; 20];

    let mut i = buf.len();
    while v > 0 {
        i -= 1;
        let digit = (v % 10) as u8;
        v /= 10;
        *unsafe { buf.get_unchecked_mut(i) } = b'0' + digit;
    }

    if i == buf.len() {
        i -= 1;
    }

    while i < buf.len() {
        unsafe { uart_write(*buf.get_unchecked(i)) };
        i += 1;
    }
}

pub fn printfv(format: &str, args: &[&dyn core::any::Any]) -> Option<()> {
    let mut argn = 0usize;
    let mut arg = false;

    for &c in format.as_bytes() {
        if c == b'%' && !arg {
            arg = true;
            continue;
        }

        if arg && c == b'd' {
            let v = *args.get(argn)?;
            argn += 1;

            let v = if let Some(&v) = v.downcast_ref::<*const u64>() {
                unsafe { *v }
            } else if let Some(&v) = v.downcast_ref::<*const i64>() {
                let v = unsafe { *v };
                if v < 0 {
                    unsafe { uart_write(b'-') };
                }

                v.abs() as u64
            } else {
                return None;
            };

            print_dec(v);
        } else if arg && c == b'x' {
            let v = *args.get(argn)?;
            argn += 1;

            let v = *v.downcast_ref::<*const u64>()?;

            print_hex(unsafe { *v });
        } else if arg && c == b'c' {
            let v = *args.get(argn)?;
            argn += 1;

            let v = *v.downcast_ref::<*const u8>()?;

            unsafe { uart_write(*v) };
        } else if arg && c == b's' {
            let v = *args.get(argn)?;
            argn += 1;

            let v = *v.downcast_ref::<*const str>()?;

            for &b in unsafe { &*v }.as_bytes() {
                unsafe { uart_write(b) };
            }
        } else if arg && c != b'%' {
            argn += 1;
        } else {
            unsafe { uart_write(c) };
        }

        arg = false;
    }

    Some(())
}

/// Supported type specifiers:
/// - %d (u64 or i64)
/// - %x (u64)
/// - %s (&str)
/// - %c (u8)
macro_rules! printf {
    ($x:literal $(,)? $($arg:expr),*) => {{
        #[allow(unused_imports)]
        use core::borrow::Borrow;
        $crate::uart::printfv($x, &[ $(&($arg.borrow() as *const _)),* ]);
    }};
}

pub(crate) use printf;
