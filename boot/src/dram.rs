use crate::{mmio, time::udelay, uart};

const CFG_SYS_SDRAM_BASE: u64 = 0x4000_0000;

const SUNXI_DRAM_TYPE_DDR2: u32 = 2;
const SUNXI_DRAM_TYPE_DDR3: u32 = 3;
const SUNXI_DRAM_TYPE_LPDDR2: u32 = 6;
const SUNXI_DRAM_TYPE_LPDDR3: u32 = 7;

const SUNXI_SID_BASE: u64 = 0x3006200;
const SUNXI_CCM_BASE: u64 = 0x2001000;

const CONFIG_DRAM_SUNXI_ODT_EN: u32 = 0x1;
const CONFIG_DRAM_SUNXI_TPR0: u32 = 0x004a2195;
const CONFIG_DRAM_SUNXI_TPR11: u32 = 0x00870000;
const CONFIG_DRAM_SUNXI_TPR12: u32 = 0x00000024;
const CONFIG_DRAM_SUNXI_TPR13: u32 = 0x34050100;
const CONFIG_DRAM_CLK: u32 = 792;
const CONFIG_DRAM_ZQ: u32 = 0x007b7bfb;
const CONFIG_SUNXI_DRAM_TYPE: u32 = 3; // DDR3

static mut DETECTED_DRAM_SIZE: u64 = 0;

struct DRAMParam {
    /* normal configuration */
    dram_clk: u32,
    dram_type: u32,
    dram_zq: u32,
    dram_odt_en: u32,

    /* timing configuration */
    dram_mr0: u32,
    dram_mr1: u32,
    dram_mr2: u32,
    dram_mr3: u32,
    dram_tpr0: u32, //DRAMTMG0
    dram_tpr1: u32, //DRAMTMG1
    dram_tpr2: u32, //DRAMTMG2
    dram_tpr3: u32, //DRAMTMG3
    dram_tpr4: u32, //DRAMTMG4
    dram_tpr5: u32, //DRAMTMG5
    dram_tpr6: u32, //DRAMTMG8
    dram_tpr7: u32,
    dram_tpr8: u32,
    dram_tpr9: u32,
    dram_tpr10: u32,
    dram_tpr11: u32,
    dram_tpr12: u32,
}

struct DRAMConfig {
    dram_para1: u32,
    dram_para2: u32,
    dram_tpr13: u32,
}

unsafe fn readl(addr: u64) -> u32 {
    unsafe { mmio::read32(addr) }
}

unsafe fn writel(v: u32, addr: u64) {
    unsafe { mmio::write32(addr, v) };
}

unsafe fn setbits_le32(addr: u64, set: u32) {
    unsafe { clrsetbits_le32(addr, 0, set) }
}

unsafe fn clrbits_le32(addr: u64, clr: u32) {
    unsafe { clrsetbits_le32(addr, clr, 0) }
}

unsafe fn clrsetbits_le32(addr: u64, clr: u32, set: u32) {
    unsafe {
        let mut v = mmio::read32(addr);
        v &= !clr;
        v |= set;
        mmio::write32(addr, v);
    }
}

fn ns_to_t(nanoseconds: u32) -> u32 {
    const CTRL_FREQ: u32 = CONFIG_DRAM_CLK / 2;

    (CTRL_FREQ * nanoseconds).div_ceil(1000)
}

unsafe fn sid_read_ldo_b_cal(para: &DRAMParam) {
    let mut reg: u32;

    reg = (unsafe { readl(SUNXI_SID_BASE + 0x1c) } & 0xff00) >> 8;

    if reg == 0 {
        return;
    }

    match para.dram_type {
        SUNXI_DRAM_TYPE_DDR2 => {}
        SUNXI_DRAM_TYPE_DDR3 => {
            if reg > 0x20 {
                reg -= 0x16;
            }
        }
        _ => {
            reg = 0;
        }
    }

    unsafe { clrsetbits_le32(0x3000150, 0xff00, reg << 8) };
}

unsafe fn dram_voltage_set(para: &DRAMParam) {
    let vol: u32;

    match para.dram_type {
        SUNXI_DRAM_TYPE_DDR2 => {
            vol = 47;
        }
        SUNXI_DRAM_TYPE_DDR3 => {
            vol = 25;
        }
        _ => {
            vol = 0;
        }
    }

    unsafe {
        clrsetbits_le32(0x3000150, 0x20ff00, vol << 8);

        udelay(1);

        sid_read_ldo_b_cal(para);
    }
}

unsafe fn dram_enable_all_master() {
    unsafe { 
        writel(!0, 0x3102020);
        writel(0xff, 0x3102024);
        writel(0xffff, 0x3102028);
        udelay(10);
    }
}

unsafe fn dram_disable_all_master() {
    unsafe { 
        writel(1, 0x3102020);
        writel(0, 0x3102024);
        writel(0, 0x3102028);
        udelay(10);
    }
}

unsafe fn bit(n: u32) -> u32 {
    1 << n
}

unsafe fn eye_delay_compensation(para: &DRAMParam) {
    let mut delay: u32;
    let mut ptr: u64;

    // DATn0IOCR, n =  0...7
    delay = (para.dram_tpr11 & 0xf) << 9;
    delay |= (para.dram_tpr12 & 0xf) << 1;
    ptr = 0x3103310;
    while ptr < 0x3103334 {
        unsafe { setbits_le32(ptr, delay) };
        ptr += 4;
    }

    // DATn1IOCR, n =  0...7
    delay = (para.dram_tpr11 & 0xf0) << 5;
    delay |= (para.dram_tpr12 & 0xf0) >> 3;
    ptr = 0x3103390;
    while ptr != 0x31033b4 {
        unsafe { setbits_le32(ptr, delay) };
        ptr += 4;
    }

    // PGCR0: assert AC loopback FIFO reset
    unsafe { clrbits_le32(0x3103100, 0x04000000) };

    // ??

    delay = (para.dram_tpr11 & 0xf0000) >> 7;
    delay |= (para.dram_tpr12 & 0xf0000) >> 15;
    unsafe { setbits_le32(0x3103334, delay) };
    unsafe { setbits_le32(0x3103338, delay) };

    delay = (para.dram_tpr11 & 0xf00000) >> 11;
    delay |= (para.dram_tpr12 & 0xf00000) >> 19;
    unsafe { setbits_le32(0x31033b4, delay) };
    unsafe { setbits_le32(0x31033b8, delay) };

    unsafe { setbits_le32(0x310333c, (para.dram_tpr11 & 0xf0000) << 9) };
    unsafe { setbits_le32(0x31033bc, (para.dram_tpr11 & 0xf00000) << 5) };

    // PGCR0: release AC loopback FIFO reset
    unsafe { setbits_le32(0x3103100, bit(26)) };

    udelay(1);

    delay = (para.dram_tpr10 & 0xf0) << 4;
    ptr = 0x3103240;
    while ptr != 0x310327c {
        unsafe { setbits_le32(ptr, delay) };
        ptr += 4;
    }

    ptr = 0x3103228;
    while ptr != 0x3103240 {
        unsafe { setbits_le32(ptr, delay) };
        ptr += 4;
    }

    unsafe { 
        setbits_le32(0x3103218, (para.dram_tpr10 & 0x0f) << 8);
        setbits_le32(0x310321c, (para.dram_tpr10 & 0x0f) << 8);
        setbits_le32(0x3103280, (para.dram_tpr10 & 0xf00) >> 4);
    }
}

/*
 * Main purpose of the auto_set_timing routine seems to be to calculate all
 * timing settings for the specific type of sdram used. Read together with
 * an sdram datasheet for context on the various variables.
 */
unsafe fn mctl_set_timing_params(para: &DRAMParam, config: &DRAMConfig) {
    /* DRAM_TPR0 */
    let tccd: u32 = 2;
    let tfaw: u32;
    let trrd: u32;
    let trcd: u32;
    let trc: u32;

    /* DRAM_TPR1 */
    let mut txp: u32;
    let twtr: u32;
    let trtp: u32 = 4;
    let twr: u32;
    let trp: u32;
    let tras: u32;

    /* DRAM_TPR2 */
    let trefi: u32;
    let trfc: u32;

    let tcksrx: u32;
    let tckesr: u32;
    let trd2wr: u32;
    let twr2rd: u32;
    let trasmax: u32;
    let twtp: u32;
    let tcke: u32;
    let tmod: u32;
    let tmrd: u32;
    let tmrw: u32;

    let tcl: u32;
    let tcwl: u32;
    let t_rdata_en: u32;
    let wr_latency: u32;

    let mr0: u32;
    let mr1: u32;
    let mr2: u32;
    let mr3: u32;

    let tdinit0: u32;
    let tdinit1: u32;
    let tdinit2: u32;
    let tdinit3: u32;

    match para.dram_type {
        SUNXI_DRAM_TYPE_DDR2 => {
            /* DRAM_TPR0 */
            tfaw = ns_to_t(50);
            trrd = ns_to_t(10);
            trcd = ns_to_t(20);
            trc = ns_to_t(65);

            /* DRAM_TPR1 */
            txp = 2;
            twtr = ns_to_t(8);
            twr = ns_to_t(15);
            trp = ns_to_t(15);
            tras = ns_to_t(45);

            /* DRAM_TRP2 */
            trfc = ns_to_t(328);
            trefi = ns_to_t(7800) / 32;

            trasmax = CONFIG_DRAM_CLK / 30;

            if CONFIG_DRAM_CLK < 409 {
                t_rdata_en = 1;
                tcl = 3;
                mr0 = 0x06a3;
            } else {
                t_rdata_en = 2;
                tcl = 4;
                mr0 = 0x0e73;
            }
            tmrd = 2;
            twtp = twr + 5;
            tcksrx = 5;
            tckesr = 4;
            trd2wr = 4;
            tcke = 3;
            tmod = 12;
            wr_latency = 1;
            tmrw = 0;
            twr2rd = twtr + 5;
            tcwl = 0;

            mr1 = para.dram_mr1;
            mr2 = 0;
            mr3 = 0;

            tdinit0 = 200 * CONFIG_DRAM_CLK + 1;
            tdinit1 = 100 * CONFIG_DRAM_CLK / 1000 + 1;
            tdinit2 = 200 * CONFIG_DRAM_CLK + 1;
            tdinit3 = 1 * CONFIG_DRAM_CLK + 1;
        }
        SUNXI_DRAM_TYPE_DDR3 => {
            trfc = ns_to_t(350);
            trefi = ns_to_t(7800) / 32 + 1; // XXX

            twtr = ns_to_t(8) + 2; // + 2 ? XXX
            trrd = ns_to_t(10).max(2);
            txp = ns_to_t(10).max(2);

            if CONFIG_DRAM_CLK <= 800 {
                tfaw = ns_to_t(50);
                trcd = ns_to_t(15);
                trp = ns_to_t(15);
                trc = ns_to_t(53);
                tras = ns_to_t(38);

                mr0 = 0x1c70;
                mr2 = 0x18;
                tcl = 6;
                wr_latency = 2;
                tcwl = 4;
                t_rdata_en = 4;
            } else {
                tfaw = ns_to_t(35);
                trcd = ns_to_t(14);
                trp = ns_to_t(14);
                trc = ns_to_t(48);
                tras = ns_to_t(34);

                mr0 = 0x1e14;
                mr2 = 0x20;
                tcl = 7;
                wr_latency = 3;
                tcwl = 5;
                t_rdata_en = 5;
            }

            trasmax = CONFIG_DRAM_CLK / 30;
            twtp = tcwl + 2 + twtr; // WL+BL/2+tWTR
                                    /* Gets overwritten below */
            //		trd2wr		= tcwl + 2 + twr;  // WL+BL/2+tWR
            twr2rd = tcwl + twtr; // WL+tWTR

            tdinit0 = 500 * CONFIG_DRAM_CLK + 1; // 500 us
            tdinit1 = 360 * CONFIG_DRAM_CLK / 1000 + 1; // 360 ns
            tdinit2 = 200 * CONFIG_DRAM_CLK + 1; // 200 us
            tdinit3 = 1 * CONFIG_DRAM_CLK + 1; //   1 us

            mr1 = para.dram_mr1;
            mr3 = 0;
            tcke = 3;
            tcksrx = 5;
            tckesr = 4;
            if ((config.dram_tpr13 & 0xc) == 0x04) || CONFIG_DRAM_CLK < 912 {
                trd2wr = 5;
            } else {
                trd2wr = 6;
            }

            tmod = 12;
            tmrd = 4;
            tmrw = 0;
        }
        SUNXI_DRAM_TYPE_LPDDR2 => {
            tfaw = ns_to_t(50).max(4);
            trrd = ns_to_t(10).max(1);
            trcd = ns_to_t(24).max(2);
            trc = ns_to_t(70);
            txp = ns_to_t(8);
            if txp < 2 {
                txp += 1;
                twtr = 2;
            } else {
                twtr = txp;
            }
            twr = ns_to_t(15).max(2);
            trp = ns_to_t(17);
            tras = ns_to_t(42);
            trefi = ns_to_t(3900) / 32;
            trfc = ns_to_t(210);

            trasmax = CONFIG_DRAM_CLK / 60;
            mr3 = para.dram_mr3;
            twtp = twr + 5;
            mr2 = 6;
            // overwritten below
            // mr1 = 5;
            tcksrx = 5;
            tckesr = 5;
            trd2wr = 10;
            tcke = 2;
            tmod = 5;
            tmrd = 5;
            tmrw = 3;
            tcl = 4;
            wr_latency = 1;
            t_rdata_en = 1;

            tdinit0 = 200 * CONFIG_DRAM_CLK + 1;
            tdinit1 = 100 * CONFIG_DRAM_CLK / 1000 + 1;
            tdinit2 = 11 * CONFIG_DRAM_CLK + 1;
            tdinit3 = 1 * CONFIG_DRAM_CLK + 1;
            twr2rd = twtr + 5;
            tcwl = 2;
            mr1 = 195;
            mr0 = 0;
        }
        SUNXI_DRAM_TYPE_LPDDR3 => {
            tfaw = ns_to_t(50).max(4);
            trrd = ns_to_t(10).max(1);
            trcd = ns_to_t(24).max(2);
            trc = ns_to_t(70);
            twtr = ns_to_t(8).max(2);
            // not used
            // twr = ns_to_t(15).max(2);
            trp = ns_to_t(17);
            tras = ns_to_t(42);
            trefi = ns_to_t(3900) / 32;
            trfc = ns_to_t(210);
            txp = twtr;

            trasmax = CONFIG_DRAM_CLK / 60;
            if CONFIG_DRAM_CLK < 800 {
                tcwl = 4;
                wr_latency = 3;
                t_rdata_en = 6;
                mr2 = 12;
            } else {
                tcwl = 3;
                // overwritten below for some reason
                // tcke = 6;
                wr_latency = 2;
                t_rdata_en = 5;
                mr2 = 10;
            }
            twtp = tcwl + 5;
            tcl = 7;
            mr3 = para.dram_mr3;
            tcksrx = 5;
            tckesr = 5;
            trd2wr = 13;
            tcke = 3;
            tmod = 12;
            tdinit0 = 400 * CONFIG_DRAM_CLK + 1;
            tdinit1 = 500 * CONFIG_DRAM_CLK / 1000 + 1;
            tdinit2 = 11 * CONFIG_DRAM_CLK + 1;
            tdinit3 = 1 * CONFIG_DRAM_CLK + 1;
            tmrd = 5;
            tmrw = 5;
            twr2rd = tcwl + twtr + 5;
            mr1 = 195;
            mr0 = 0;
        }
        _ => {
            trfc = 128;
            trp = 6;
            trefi = 98;
            txp = 10;
            /*
            not used:
            twr = 8;
            twtr = 3;
            */
            tras = 14;
            tfaw = 16;
            trc = 20;
            trcd = 6;
            trrd = 3;

            twr2rd = 8;
            tcksrx = 4;
            tckesr = 3;
            trd2wr = 4;
            trasmax = 27;
            twtp = 12;
            tcke = 2;
            tmod = 6;
            tmrd = 2;
            tmrw = 0;
            tcwl = 3;
            tcl = 3;
            wr_latency = 1;
            t_rdata_en = 1;
            mr3 = 0;
            mr2 = 0;
            mr1 = 0;
            mr0 = 0;
            tdinit3 = 0;
            tdinit2 = 0;
            tdinit1 = 0;
            tdinit0 = 0;
        }
    }

    unsafe {
        /* Set mode registers */
        writel(mr0, 0x3103030);
        writel(mr1, 0x3103034);
        writel(mr2, 0x3103038);
        writel(mr3, 0x310303c);
        /* TODO: dram_odt_en is either 0x0 or 0x1, so right shift looks weird */
        writel((para.dram_odt_en >> 4) & 0x3, 0x310302c);

        /* Set dram timing DRAMTMG0 - DRAMTMG5 */
        writel(
            ((twtp << 24) | (tfaw << 16) | (trasmax << 8) | (tras << 0)) as u32,
            0x3103058,
        );
        writel(((txp << 16) | (trtp << 8) | (trc << 0)) as u32, 0x310305c);
        writel(
            ((tcwl << 24) | (tcl << 16) | (trd2wr << 8) | (twr2rd << 0)) as u32,
            0x3103060,
        );
        writel(
            ((tmrw << 16) | (tmrd << 12) | (tmod << 0)) as u32,
            0x3103064,
        );
        writel(
            ((trcd << 24) | (tccd << 16) | (trrd << 8) | (trp << 0)) as u32,
            0x3103068,
        );
        writel(
            ((tcksrx << 24) | (tcksrx << 16) | (tckesr << 8) | (tcke << 0)) as u32,
            0x310306c,
        );

        /* Set dual rank timing */
        clrsetbits_le32(
            0x3103078,
            0xf000ffff,
            if CONFIG_DRAM_CLK < 800 {
                0xf0006610
            } else {
                0xf0007610
            },
        );

        /* Set phy interface time PITMG0, PTR3, PTR4 */
        writel(
            (0x2 << 24) as u32 | (t_rdata_en << 16) as u32 | bit(8) | ((wr_latency as u32) << 0),
            0x3103080,
        );
        writel((tdinit0 << 0) | (tdinit1 << 20), 0x3103050);
        writel((tdinit2 << 0) | (tdinit3 << 20), 0x3103054);

        /* Set refresh timing and mode */
        writel((trefi << 16) | (trfc << 0), 0x3103090);
        writel((trefi << 15) & 0x0fff0000, 0x3103094);
    }
}

// Purpose of this routine seems to be to initialize the PLL driving
// the MBUS and sdram.
//
unsafe fn ccu_set_pll_ddr_clk(para: &DRAMParam, config: &DRAMConfig) -> u32 {
    unsafe { 
    let mut val: u32;
    let clk: u32;
    let n: u32;

    if (config.dram_tpr13 & bit(6)) > 0 {
        clk = para.dram_tpr9;
    } else {
        clk = para.dram_clk;
    }

    // set VCO clock divider
    n = (clk * 2) / 24;

    val = readl(SUNXI_CCM_BASE + 0x10);
    val &= !0x0007ff03; // clear dividers
    val |= (n - 1) << 8; // set PLL division
    val |= bit(31) | bit(30); // enable PLL and LDO
    writel(val | bit(29), SUNXI_CCM_BASE + 0x10);

    // wait for PLL to lock
    while (readl(SUNXI_CCM_BASE + 0x10) & bit(28)) == 0 {}

    udelay(20);

    // enable PLL output
    setbits_le32(SUNXI_CCM_BASE + 0x0, bit(27));

    // turn clock gate on
    val = readl(SUNXI_CCM_BASE + 0x800);
    val &= !0x03000303; // select DDR clk source, n=1, m=1
    val |= bit(31); // turn clock on
    writel(val, SUNXI_CCM_BASE + 0x800);

    n * 24
    }
}

/* Set up the PLL and clock gates for the DRAM controller and MBUS clocks. */
unsafe fn mctl_sys_init(para: &DRAMParam, config: &DRAMConfig) {
    unsafe { 
    // assert MBUS reset
    clrbits_le32(SUNXI_CCM_BASE + 0x540, bit(30));

    // turn off sdram clock gate, assert sdram reset
    clrbits_le32(SUNXI_CCM_BASE + 0x80c, 0x10001);
    clrsetbits_le32(SUNXI_CCM_BASE + 0x800, bit(31) | bit(30), bit(27));
    udelay(10);

    // set ddr pll clock
    ccu_set_pll_ddr_clk(para, config);
    udelay(100);
    dram_disable_all_master();

    // release sdram reset
    setbits_le32(SUNXI_CCM_BASE + 0x80c, bit(16));

    // release MBUS reset
    setbits_le32(SUNXI_CCM_BASE + 0x540, bit(30));
    setbits_le32(SUNXI_CCM_BASE + 0x800, bit(30));

    udelay(5);

    // turn on sdram clock gate
    setbits_le32(SUNXI_CCM_BASE + 0x80c, bit(0));

    // turn dram clock gate on, trigger sdr clock update
    setbits_le32(SUNXI_CCM_BASE + 0x800, bit(31) | bit(27));
    udelay(5);

    // mCTL clock enable
    writel(0x8000, 0x310300c);
    udelay(10);
    }
}

// The main purpose of this routine seems to be to copy an address configuration
// from the dram_para1 and dram_para2 fields to the PHY configuration registers
// (0x3102000, 0x3102004).
//
unsafe fn mctl_com_init(para: &DRAMParam, config: &DRAMConfig) {
    unsafe { 
    let mut val: u32;
    let width: u32;
    let mut ptr: u64;
    let mut i: u32;

    // purpose ??
    clrsetbits_le32(0x3102008, 0x3f00, 0x2000);

    // set SDRAM type and word width
    val = readl(0x3102000) & !0x00fff000;
    val |= (para.dram_type & 0x7) << 16; // DRAM type
    val |= (!config.dram_para2 & 0x1) << 12; // DQ width
    val |= bit(22); // ??
    if para.dram_type == SUNXI_DRAM_TYPE_LPDDR2 || para.dram_type == SUNXI_DRAM_TYPE_LPDDR3 {
        val |= bit(19); // type 6 and 7 must use 1T
    } else {
        if (config.dram_tpr13 & bit(5)) > 0 {
            val |= bit(19);
        }
    }
    writel(val, 0x3102000);

    // init rank / bank / row for single/dual or two different ranks
    if (config.dram_para2 & bit(8)) > 0 && ((config.dram_para2 & 0xf000) != 0x1000) {
        width = 32;
    } else {
        width = 16;
    }

    ptr = 0x3102000;
    i = 0;
    while i < width {
        val = readl(ptr) & 0xfffff000;

        val |= (config.dram_para2 >> 12) & 0x3; // rank
        val |= ((config.dram_para1 >> (i + 12)) << 2) & 0x4; // bank - 2
        val |= (((config.dram_para1 >> (i + 4)) - 1) << 4) & 0xff; // row - 1

        // convert from page size to column addr width - 3
        match (config.dram_para1 >> i) & 0xf {
            8 => val |= 0xa00,
            4 => val |= 0x900,
            2 => val |= 0x800,
            1 => val |= 0x700,
            _ => val |= 0x600,
        }
        writel(val, ptr);
        ptr += 4;
        i += 16;
    }

    // set ODTMAP based on number of ranks in use
    val = if (readl(0x3102000) & 0x1) > 0 {
        0x303
    } else {
        0x201
    };
    writel(val, 0x3103120);

    // set mctl reg 3c4 to zero when using half DQ
    if config.dram_para2 & bit(0) > 0 {
        writel(0, 0x31033c4);
    }

    // purpose ??
    if para.dram_tpr4 > 0 {
        setbits_le32(0x3102000, (para.dram_tpr4 & 0x3) << 25);
        setbits_le32(0x3102004, (para.dram_tpr4 & 0x7fc) << 10);
    }
    }
}

const AC_REMAPPING_TABLES: [[u8; 22]; 8] = [
    [0; 22],
    [
        1, 9, 3, 7, 8, 18, 4, 13, 5, 6, 10, 2, 14, 12, 0, 0, 21, 17, 20, 19, 11, 22,
    ],
    [
        4, 9, 3, 7, 8, 18, 1, 13, 2, 6, 10, 5, 14, 12, 0, 0, 21, 17, 20, 19, 11, 22,
    ],
    [
        1, 7, 8, 12, 10, 18, 4, 13, 5, 6, 3, 2, 9, 0, 0, 0, 21, 17, 20, 19, 11, 22,
    ],
    [
        4, 12, 10, 7, 8, 18, 1, 13, 2, 6, 3, 5, 9, 0, 0, 0, 21, 17, 20, 19, 11, 22,
    ],
    [
        13, 2, 7, 9, 12, 19, 5, 1, 6, 3, 4, 8, 10, 0, 0, 0, 21, 22, 18, 17, 11, 20,
    ],
    [
        3, 10, 7, 13, 9, 11, 1, 2, 4, 6, 8, 5, 12, 0, 0, 0, 20, 1, 0, 21, 22, 17,
    ],
    [
        3, 2, 4, 7, 9, 1, 17, 12, 18, 14, 13, 8, 15, 6, 10, 5, 19, 22, 16, 21, 20, 11,
    ],
];

/*
 * This routine chooses one of several remapping tables for 22 lines.
 * It is unclear which lines are being remapped. It seems to pick
 * table cfg7 for the Nezha board.
 */
unsafe fn mctl_phy_ac_remapping(para: &DRAMParam, config: &DRAMConfig) {
    unsafe { 
    let cfg: &[u8; 22];
    let fuse: u32;
    let mut val: u32;

    /*
     * It is unclear whether the LPDDRx types don't need any remapping,
     * or whether the original code just didn't provide tables.
     */
    if para.dram_type != SUNXI_DRAM_TYPE_DDR2 && para.dram_type != SUNXI_DRAM_TYPE_DDR3 {
        return;
    }

    fuse = (readl(SUNXI_SID_BASE + 0x28) & 0xf00) >> 8;
    uart::printf!("DDR efuse: 0x%x\r\n", fuse);

    if para.dram_type == SUNXI_DRAM_TYPE_DDR2 {
        if fuse == 15 {
            return;
        }
        cfg = &AC_REMAPPING_TABLES[6];
    } else {
        if (config.dram_tpr13 & 0xc0000) > 0 {
            cfg = &AC_REMAPPING_TABLES[7];
        } else {
            match fuse {
                8 => cfg = &AC_REMAPPING_TABLES[2],
                9 => cfg = &AC_REMAPPING_TABLES[3],
                10 => cfg = &AC_REMAPPING_TABLES[5],
                11 => cfg = &AC_REMAPPING_TABLES[4],
                13 | 14 => cfg = &AC_REMAPPING_TABLES[0],
                12 | _ => cfg = &AC_REMAPPING_TABLES[1],
            }
        }
    }

    val = ((cfg[4] as u32) << 25)
        | ((cfg[3] as u32) << 20)
        | ((cfg[2] as u32) << 15)
        | ((cfg[1] as u32) << 10)
        | ((cfg[0] as u32) << 5);
    writel(val, 0x3102500);

    val = ((cfg[10] as u32) << 25)
        | ((cfg[9] as u32) << 20)
        | ((cfg[8] as u32) << 15)
        | ((cfg[7] as u32) << 10)
        | ((cfg[6] as u32) << 5)
        | cfg[5] as u32;
    writel(val, 0x3102504);

    val = ((cfg[15] as u32) << 20)
        | ((cfg[14] as u32) << 15)
        | ((cfg[13] as u32) << 10)
        | ((cfg[12] as u32) << 5)
        | cfg[11] as u32;
    writel(val, 0x3102508);

    val = ((cfg[21] as u32) << 25)
        | ((cfg[20] as u32) << 20)
        | ((cfg[19] as u32) << 15)
        | ((cfg[18] as u32) << 10)
        | ((cfg[17] as u32) << 5)
        | cfg[16] as u32;
    writel(val, 0x310250c);

    val = ((cfg[4] as u32) << 25)
        | ((cfg[3] as u32) << 20)
        | ((cfg[2] as u32) << 15)
        | ((cfg[1] as u32) << 10)
        | ((cfg[0] as u32) << 5)
        | 1;

    writel(val, 0x3102500);
    }
}

// Init the controller channel. The key part is placing commands in the main
// command register (PIR, 0x3103000) and checking command status (PGSR0, 0x3103010).
//
unsafe fn mctl_channel_init(para: &DRAMParam, config: &DRAMConfig) -> bool {
    unsafe { 
    let val: u32;
    let dqs_gating_mode: u32;

    dqs_gating_mode = (config.dram_tpr13 & 0xc) >> 2;

    // set DDR clock to half of CPU clock
    clrsetbits_le32(0x310200c, 0xfff, (para.dram_clk / 2) - 1);

    // MRCTRL0 nibble 3 undocumented
    clrsetbits_le32(0x3103108, 0xf00, 0x300);

    if para.dram_odt_en > 0 {
        val = 0;
    } else {
        val = bit(5);
    }

    // DX0GCR0
    if para.dram_clk > 672 {
        clrsetbits_le32(0x3103344, 0xf63e, val);
    } else {
        clrsetbits_le32(0x3103344, 0xf03e, val);
    }

    // DX1GCR0
    if para.dram_clk > 672 {
        setbits_le32(0x3103344, 0x400);
        clrsetbits_le32(0x31033c4, 0xf63e, val);
    } else {
        clrsetbits_le32(0x31033c4, 0xf03e, val);
    }

    // 0x3103208 undocumented
    setbits_le32(0x3103208, bit(1));

    eye_delay_compensation(para);

    // set PLL SSCG ?
    // does reading and discarding here have some side effects ?
    let _val = readl(0x3103108);
    if dqs_gating_mode == 1 {
        clrsetbits_le32(0x3103108, 0xc0, 0);
        clrbits_le32(0x31030bc, 0x107);
    } else if dqs_gating_mode == 2 {
        clrsetbits_le32(0x3103108, 0xc0, 0x80);

        clrsetbits_le32(
            0x31030bc,
            0x107,
            (((config.dram_tpr13 >> 16) & 0x1f) - 2) | 0x100,
        );
        clrsetbits_le32(0x310311c, bit(31), bit(27));
    } else {
        clrbits_le32(0x3103108, 0x40);
        udelay(10);
        setbits_le32(0x3103108, 0xc0);
    }

    if para.dram_type == SUNXI_DRAM_TYPE_LPDDR2 || para.dram_type == SUNXI_DRAM_TYPE_LPDDR3 {
        if dqs_gating_mode == 1 {
            clrsetbits_le32(0x310311c, 0x080000c0, 0x80000000);
        } else {
            clrsetbits_le32(0x310311c, 0x77000000, 0x22000000);
        }
    }

    clrsetbits_le32(
        0x31030c0,
        0x0fffffff,
        if (config.dram_para2 & bit(12)) > 0 {
            0x03000001
        } else {
            0x01000007
        },
    );

    if (readl(0x70005d4) & bit(16)) > 0 {
        clrbits_le32(0x7010250, 0x2);
        udelay(10);
    }

    // Set ZQ config
    clrsetbits_le32(0x3103140, 0x3ffffff, (para.dram_zq & 0x00ffffff) | bit(25));

    // Initialise DRAM controller
    if dqs_gating_mode == 1 {
        //writel(0x52, 0x3103000); // prep PHY reset + PLL init + z-cal
        writel(0x53, 0x3103000); // Go

        while (readl(0x3103010) & 0x1) == 0 {} // wait for IDONE
        udelay(10);

        // 0x520 = prep DQS gating + DRAM init + d-cal
        if para.dram_type == SUNXI_DRAM_TYPE_DDR3 {
            writel(0x5a0, 0x3103000); // + DRAM reset
        } else {
            writel(0x520, 0x3103000);
        }
    } else {
        if (readl(0x70005d4) & (1 << 16)) == 0 {
            // prep DRAM init + PHY reset + d-cal + PLL init + z-cal
            if para.dram_type == SUNXI_DRAM_TYPE_DDR3 {
                writel(0x1f2, 0x3103000); // + DRAM reset
            } else {
                writel(0x172, 0x3103000);
            }
        } else {
            // prep PHY reset + d-cal + z-cal
            writel(0x62, 0x3103000);
        }
    }

    setbits_le32(0x3103000, 0x1); // GO

    udelay(10);
    while (readl(0x3103010) & 0x1) == 0 {} // wait for IDONE

    if (readl(0x70005d4) & bit(16)) > 0 {
        clrsetbits_le32(0x310310c, 0x06000000, 0x04000000);
        udelay(10);

        setbits_le32(0x3103004, 0x1);

        while (readl(0x3103018) & 0x7) != 0x3 {}

        clrbits_le32(0x7010250, 0x1);
        udelay(10);

        clrbits_le32(0x3103004, 0x1);

        while (readl(0x3103018) & 0x7) != 0x1 {}

        udelay(15);

        if dqs_gating_mode == 1 {
            clrbits_le32(0x3103108, 0xc0);
            clrsetbits_le32(0x310310c, 0x06000000, 0x02000000);
            udelay(1);
            writel(0x401, 0x3103000);

            while (readl(0x3103010) & 0x1) == 0 {}
        }
    }

    // Check for training error
    if (readl(0x3103010) & bit(20)) > 0 {
        uart::printf!("ZQ calibration error, check external 240 ohm resistor\r\n");
        return false;
    }

    // STATR = Zynq STAT? Wait for status 'normal'?
    while (readl(0x3103018) & 0x1) == 0 {}

    setbits_le32(0x310308c, bit(31));
    udelay(10);
    clrbits_le32(0x310308c, bit(31));
    udelay(10);
    setbits_le32(0x3102014, bit(31));
    udelay(10);

    clrbits_le32(0x310310c, 0x06000000);

    if dqs_gating_mode == 1 {
        clrsetbits_le32(0x310311c, 0xc0, 0x40);
    }

    true
    }
}

unsafe fn calculate_rank_size(regval: u32) -> u32 {
    let mut bits: u32;

    bits = (regval >> 8) & 0xf; /* page size - 3 */
    bits += (regval >> 4) & 0xf; /* row width - 1 */
    bits += (regval >> 2) & 0x3; /* bank count - 2 */
    bits -= 14; /* 1MB = 20 bits, minus above 6 = 14 */

    return 1 << bits;
}

/*
 * The below routine reads the dram config registers and extracts
 * the number of address bits in each rank available. It then calculates
 * total memory size in MB.
 */
unsafe fn dramc_get_dram_size() -> u32 {
    unsafe { 
    let mut val: u32;
    let size: u32;

    val = readl(0x3102000); /* MC_WORK_MODE0 */
    size = calculate_rank_size(val);
    if (val & 0x3) == 0 {
        /* single rank? */
        return size;
    }

    val = readl(0x3102004); /* MC_WORK_MODE1 */
    if (val & 0x3) == 0 {
        /* two identical ranks? */
        return size * 2;
    }

    /* add sizes of both ranks */
    return size + calculate_rank_size(val);
    }
}

/*
 * The below routine reads the command status register to extract
 * DQ width and rank count. This follows the DQS training command in
 * channel_init. If error bit 22 is reset, we have two ranks and full DQ.
 * If there was an error, figure out whether it was half DQ, single rank,
 * or both. Set bit 12 and 0 in dram_para2 with the results.
 */
unsafe fn dqs_gate_detect(config: &mut DRAMConfig) -> bool {
    unsafe { 
    let dx0: u32;
    let mut dx1: u32 = 0;

    if (readl(0x3103010) & bit(22)) == 0 {
        config.dram_para2 = (config.dram_para2 & !0xf) | bit(12);
        uart::printf!("dual rank and full DQ\r\n");

        return true;
    }

    dx0 = (readl(0x3103348) & 0x3000000) >> 24;
    if dx0 == 0 {
        config.dram_para2 = (config.dram_para2 & !0xf) | 0x1001;
        uart::printf!("dual rank and half DQ\r\n");

        return true;
    }

    if dx0 == 2 {
        dx1 = (readl(0x31033c8) & 0x3000000) >> 24;
        if dx1 == 2 {
            config.dram_para2 = config.dram_para2 & !0xf00f;
            uart::printf!("single rank and full DQ\r\n");
        } else {
            config.dram_para2 = (config.dram_para2 & !0xf00f) | bit(0);
            uart::printf!("single rank and half DQ\r\n");
        }

        return true;
    }

    if (config.dram_tpr13 & bit(29)) == 0 {
        return false;
    }

    uart::printf!("DX0 state: %d\r\n", dx0);
    uart::printf!("DX1 state: %d\r\n", dx1);

    return false;
    }
}

unsafe fn dramc_simple_wr_test(mem_mb: u32, len: u32) -> bool {
    unsafe { 
    let offs = ((mem_mb / 2) << 18) as u64; // half of memory size
    let patt1: u32 = 0x01234567;
    let patt2: u32 = 0xfedcba98;
    let mut addr: u64;
    let mut v1: u32;
    let mut v2: u32;

    addr = CFG_SYS_SDRAM_BASE;
    for i in 0..len {
        writel(patt1 + i, addr);
        writel(patt2 + i, addr + offs);
        addr += 4;
    }

    addr = CFG_SYS_SDRAM_BASE;
    for i in 0..len {
        v1 = readl(addr);
        v2 = patt1 + i;
        if v1 != v2 {
            uart::printf!("DRAM: simple test FAIL\r\n");
            uart::printf!("%x != %x at address %x\r\n", v1, v2, addr);
            return true;
        }
        v1 = readl(addr + offs);
        v2 = patt2 + i;
        if v1 != v2 {
            uart::printf!("DRAM: simple test FAIL\r\n");
            uart::printf!("%x != %x at address %x\r\n", v1, v2, addr + offs);
            return true;
        }

        addr += 4;
    }

    uart::printf!("DRAM: simple test OK\r\n");
    false
    }
}

// Set the Vref mode for the controller
//
unsafe fn mctl_vrefzq_init(para: &DRAMParam, config: &DRAMConfig) {
    unsafe { 
    if (config.dram_tpr13 & bit(17)) > 0 {
        return;
    }

    clrsetbits_le32(0x3103110, 0x7f7f7f7f, para.dram_tpr5);

    // IOCVR1
    if (config.dram_tpr13 & bit(16)) == 0 {
        clrsetbits_le32(0x3103114, 0x7f, para.dram_tpr6 & 0x7f);
    }
    }
}

// Perform an init of the controller. This is actually done 3 times. The first
// time to establish the number of ranks and DQ width. The second time to
// establish the actual ram size. The third time is final one, with the final
// settings.
//
unsafe fn mctl_core_init(para: &DRAMParam, config: &DRAMConfig) -> bool {
    unsafe { 
    mctl_sys_init(para, config);

    mctl_vrefzq_init(para, config);

    mctl_com_init(para, config);

    mctl_phy_ac_remapping(para, config);

    mctl_set_timing_params(para, config);

    return mctl_channel_init(para, config);
    }
}

/*
 * This routine sizes a DRAM device by cycling through address lines and
 * figuring out if they are connected to a real address line, or if the
 * address is a mirror.
 * First the column and bank bit allocations are set to low values (2 and 9
 * address lines). Then a maximum allocation (16 lines) is set for rows and
 * this is tested.
 * Next the BA2 line is checked. This seems to be placed above the column,
 * BA0-1 and row addresses. Finally, the column address is allocated 13 lines
 * and these are tested. The results are placed in dram_para1 and dram_para2.
 */

fn get_payload(odd: bool, ptr: u64) -> u32 {
    if odd {
        return ptr as u32;
    } else {
        return !(ptr as u32);
    }
}

unsafe fn auto_scan_dram_size(para: &DRAMParam, config: &mut DRAMConfig) -> bool {
    unsafe { 
    let mut rval: u32 = 0;
    let mut i: u32;
    let mut j: u32;
    let mut rank: u32;
    let maxrank: u32;
    let mut offs: u32;
    let mut shft: u32;
    let mut ptr: u64;
    let mut mc_work_mode: u64;
    let mut chk: u64;

    if !mctl_core_init(para, config) {
        uart::printf!("DRAM initialisation error : 0\r\n");
        return false;
    }

    maxrank = if (config.dram_para2 & 0xf000) > 0 {
        2
    } else {
        1
    };
    mc_work_mode = 0x3102000;
    offs = 0;

    /* write test pattern */
    i = 0;
    ptr = CFG_SYS_SDRAM_BASE;
    while i < 64 {
        writel(get_payload((i & 0x1) > 0, ptr), ptr);
        ptr += 4;
        i += 1;
    }

    rank = 0;
    while rank < maxrank {
        /* set row mode */
        clrsetbits_le32(mc_work_mode, 0xf0c, 0x6f0);
        udelay(1);

        // Scan per address line, until address wraps (i.e. see shadow)
        i = 11;
        while i < 17 {
            chk = CFG_SYS_SDRAM_BASE + (1 << (i + 11));
            ptr = CFG_SYS_SDRAM_BASE;
            j = 0;

            while j < 64 {
                if readl(chk) != get_payload((j & 0x1) > 0, ptr) {
                    break;
                }
                ptr += 4;
                chk += 4;
                j += 1;
            }

            if j == 64 {
                break;
            }

            i += 1;
        }

        if i > 16 {
            i = 16;
        }

        uart::printf!("rank %d row = %d\r\n", rank, i);

        /* Store rows in para 1 */
        shft = offs + 4;
        rval = config.dram_para1;
        rval &= !(0xff << shft);
        rval |= i << shft;
        config.dram_para1 = rval;

        if rank == 1
        /* Set bank mode for rank0 */
        {
            clrsetbits_le32(0x3102000, 0xffc, 0x6a4);
        }

        /* Set bank mode for current rank */
        clrsetbits_le32(mc_work_mode, 0xffc, 0x6a4);
        udelay(1);

        // Test if bit A23 is BA2 or mirror XXX A22?
        chk = CFG_SYS_SDRAM_BASE + (1 << 22);
        ptr = CFG_SYS_SDRAM_BASE;
        i = 0;
        j = 0;
        while i < 64 {
            if readl(chk) != get_payload((i & 1) > 0, ptr) {
                j = 1;
                break;
            }
            ptr += 4;
            chk += 4;
            i += 1;
        }

        uart::printf!("rank %d bank = %d\r\n", rank, (j + 1) << 2); /* 4 or 8 */

        /* Store banks in para 1 */
        shft = 12 + offs;
        rval = config.dram_para1;
        rval &= !(0xf << shft);
        rval |= j << shft;
        config.dram_para1 = rval;

        if rank == 1
        /* Set page mode for rank0 */
        {
            clrsetbits_le32(0x3102000, 0xffc, 0xaa0);
        }

        /* Set page mode for current rank */
        clrsetbits_le32(mc_work_mode, 0xffc, 0xaa0);
        udelay(1);

        // Scan per address line, until address wraps (i.e. see shadow)
        i = 9;
        while i < 14 {
            chk = CFG_SYS_SDRAM_BASE + (1 << i);
            ptr = CFG_SYS_SDRAM_BASE;
            j = 0;
            while j < 64 {
                if readl(chk) != get_payload((j & 1) > 0, ptr) {
                    break;
                }
                ptr += 4;
                chk += 4;
                j += 1;
            }
            if j == 64 {
                break;
            }
            i += 1;
        }
        if i > 13 {
            i = 13;
        }

        let pgsize = if i == 9 { 0 } else { 1 << (i - 10) };

        uart::printf!("rank %d page size = %d KB\r\n", rank, pgsize);

        /* Store page size */
        shft = offs;
        rval = config.dram_para1;
        rval &= !(0xf << shft);
        rval |= pgsize << shft;
        config.dram_para1 = rval;

        // Move to next rank
        rank += 1;
        if rank != maxrank {
            if rank == 1 {
                /* MC_WORK_MODE */
                clrsetbits_le32(0x3202000, 0xffc, 0x6f0);

                /* MC_WORK_MODE2 */
                clrsetbits_le32(0x3202004, 0xffc, 0x6f0);
            }
            /* store rank1 config in upper half of para1 */
            offs += 16;
            mc_work_mode += 4; /* move to MC_WORK_MODE2 */
        }
    }
    if maxrank == 2 {
        config.dram_para2 &= 0xfffff0ff;
        /* note: rval is equal to para->dram_para1 here */
        if (rval & 0xffff) == (rval >> 16) {
            uart::printf!("rank1 config same as rank0\r\n");
        } else {
            config.dram_para2 |= bit(8);
            uart::printf!("rank1 config different from rank0\r\n");
        }
    }

    return true;
    }
}

/*
 * This routine sets up parameters with dqs_gating_mode equal to 1 and two
 * ranks enabled. It then configures the core and tests for 1 or 2 ranks and
 * full or half DQ width. It then resets the parameters to the original values.
 * dram_para2 is updated with the rank and width findings.
 */
unsafe fn auto_scan_dram_rank_width(para: &DRAMParam, config: &mut DRAMConfig) -> bool {
    unsafe {
    let s1: u32 = config.dram_tpr13;
    let s2: u32 = config.dram_para1;

    config.dram_para1 = 0x00b000b0;
    config.dram_para2 = (config.dram_para2 & !0xf) | bit(12);

    /* set DQS probe mode */
    config.dram_tpr13 = (config.dram_tpr13 & !0x8) | bit(2) | bit(0);

    mctl_core_init(para, config);

    if (readl(0x3103010) & bit(20)) > 0 {
        return false;
    }

    if !dqs_gate_detect(config) {
        return false;
    }

    config.dram_tpr13 = s1;
    config.dram_para1 = s2;

    return true;
    }
}

/*
 * This routine determines the SDRAM topology. It first establishes the number
 * of ranks and the DQ width. Then it scans the SDRAM address lines to establish
 * the size of each rank. It then updates dram_tpr13 to reflect that the sizes
 * are now known: a re-init will not repeat the autoscan.
 */
unsafe fn auto_scan_dram_config(para: &DRAMParam, config: &mut DRAMConfig) -> bool {
    unsafe { 
    if ((config.dram_tpr13 & bit(14)) == 0) && (!auto_scan_dram_rank_width(para, config)) {
        uart::printf!("ERROR: auto scan dram rank & width failed\r\n");
        return false;
    }

    if ((config.dram_tpr13 & bit(0)) == 0) && (!auto_scan_dram_size(para, config)) {
        uart::printf!("ERROR: auto scan dram size failed\r\n");
        return false;
    }

    if (config.dram_tpr13 & bit(15)) == 0 {
        config.dram_tpr13 |= bit(14) | bit(13) | bit(1) | bit(0);
    }

    return true;
    }
}

unsafe fn do_init_dram(para: &DRAMParam) -> Option<u32> {
    unsafe { 
    let mut config = DRAMConfig {
        dram_para1: 0x000010d2,
        dram_para2: 0,
        dram_tpr13: CONFIG_DRAM_SUNXI_TPR13,
    };

    let mut rc: u32;
    let mem_size_mb: u32;

    uart::printf!("DRAM CLK = %d MHz\r\n", para.dram_clk);
    uart::printf!("DRAM Type = %d\r\n", para.dram_type);

    if (para.dram_odt_en & 0x1) == 0 {
        uart::printf!("DRAMC read ODT off\r\n");
    } else {
        uart::printf!("DRAMC ZQ value: 0x%x\r\n", para.dram_zq);
    }

    /* Test ZQ status */
    if (config.dram_tpr13 & bit(16)) > 0 {
        uart::printf!("DRAM only have internal ZQ\r\n");
        setbits_le32(0x3000160, bit(8));
        writel(0, 0x3000168);
        udelay(10);
    } else {
        clrbits_le32(0x3000160, 0x3);
        writel(config.dram_tpr13 & bit(16), 0x7010254);
        udelay(10);
        clrsetbits_le32(0x3000160, 0x108, bit(1));
        udelay(10);
        setbits_le32(0x3000160, bit(0));
        udelay(20);
        uart::printf!("ZQ value = 0x%x\r\n", readl(0x300016c));
    }

    dram_voltage_set(para);

    /* Set SDRAM controller auto config */
    if (config.dram_tpr13 & bit(0)) == 0 {
        if !auto_scan_dram_config(para, &mut config) {
            uart::printf!("auto_scan_dram_config() FAILED\r\n");
            return None;
        }
    }

    /* report ODT */
    rc = para.dram_mr1;
    if (rc & 0x44) == 0 {
        uart::printf!("DRAM ODT off\r\n");
    } else {
        uart::printf!("DRAM ODT value: 0x%x\r\n", rc);
    }

    /* Init core, final run */
    if !mctl_core_init(para, &config) {
        uart::printf!("DRAM initialisation error: 1\r\n");
        return None;
    }

    /* Get SDRAM size */
    /* TODO: who ever puts a negative number in the top half? */
    rc = config.dram_para2;
    if (rc & bit(31)) > 0 {
        rc = (rc >> 16) & !bit(15);
    } else {
        rc = dramc_get_dram_size();
        uart::printf!("DRAM: size = %dMB\r\n", rc);
        config.dram_para2 = (config.dram_para2 & 0xffff) | rc << 16;
    }
    mem_size_mb = rc;

    /* Purpose ?? */
    if (config.dram_tpr13 & bit(30)) > 0 {
        rc = para.dram_tpr8;
        if rc == 0 {
            rc = 0x10000200;
        }
        writel(rc, 0x31030a0);
        writel(0x40a, 0x310309c);
        setbits_le32(0x3103004, bit(0));
        uart::printf!("Enable Auto SR\r\n");
    } else {
        clrbits_le32(0x31030a0, 0xffff);
        clrbits_le32(0x3103004, 0x1);
    }

    /* Purpose ?? */
    if (config.dram_tpr13 & bit(9)) > 0 {
        clrsetbits_le32(0x3103100, 0xf000, 0x5000);
    } else {
        if para.dram_type != SUNXI_DRAM_TYPE_LPDDR2 {
            clrbits_le32(0x3103100, 0xf000);
        }
    }

    setbits_le32(0x3103140, bit(31));

    /* CHECK: is that really writing to a different register? */
    if (config.dram_tpr13 & bit(8)) > 0 {
        writel(readl(0x3103140) | 0x300, 0x31030b8);
    }

    if (config.dram_tpr13 & bit(16)) > 0 {
        clrbits_le32(0x3103108, bit(13));
    } else {
        setbits_le32(0x3103108, bit(13));
    }

    /* Purpose ?? */
    if para.dram_type == SUNXI_DRAM_TYPE_LPDDR3 {
        clrsetbits_le32(0x310307c, 0xf0000, 0x1000);
    }

    dram_enable_all_master();
    if (config.dram_tpr13 & bit(28)) > 0 {
        if (readl(0x70005d4) & bit(16)) > 0 || dramc_simple_wr_test(mem_size_mb, 4096) {
            return None;
        }
    }

    uart::printf!("initialized DRAM: memory size = %d MB\r\n", mem_size_mb);

    return Some(mem_size_mb);
    }
}

pub unsafe fn init_dram() {
    let para = DRAMParam {
        dram_clk: CONFIG_DRAM_CLK,
        dram_type: CONFIG_SUNXI_DRAM_TYPE,
        dram_zq: CONFIG_DRAM_ZQ,
        dram_odt_en: CONFIG_DRAM_SUNXI_ODT_EN,
        dram_mr0: 0x1c70,
        dram_mr1: 0x42,
        dram_mr2: 0x18,
        dram_mr3: 0,
        dram_tpr0: 0x004a2195,
        dram_tpr1: 0x02423190,
        dram_tpr2: 0x0008b061,
        dram_tpr3: 0xb4787896, // unused
        dram_tpr4: 0,
        dram_tpr5: 0x48484848,
        dram_tpr6: 0x00000048,
        dram_tpr7: 0x1620121e, // unused
        dram_tpr8: 0,
        dram_tpr9: 0, // clock?
        dram_tpr10: 0,
        dram_tpr11: CONFIG_DRAM_SUNXI_TPR11,
        dram_tpr12: CONFIG_DRAM_SUNXI_TPR12,
    };

    let Some(size_mb) = (unsafe{ do_init_dram(&para) }) else {
        uart::printf!("failed to initialize DRAM\r\n");
        loop {}
    };

    unsafe { DETECTED_DRAM_SIZE = size_mb as u64 * 1024 * 1024 };

    uart::printf!("initialized DRAM: %d MB at 0x%x\r\n", size_mb, CFG_SYS_SDRAM_BASE);
}

pub fn dram_size() -> u64 {
    unsafe { DETECTED_DRAM_SIZE }
}

pub fn dram_base() -> *mut u8 {
    CFG_SYS_SDRAM_BASE as *mut u8
}
