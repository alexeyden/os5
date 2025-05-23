use crate::mmio::*;
use crate::time::*;

const CCU_BASE: u64 = 0x02001000;

const CCU_PLL_CPU_CTRL: u64 = 0x0000;
const CCU_PLL_PERI_CTRL: u64 = 0x0020;
const CCU_CPU_AXI_CFG: u64 = 0x0500;
const CCU_PSI_CLK: u64 = 0x0510;
const CCU_APB0_CLK: u64 = 0x0520;
const CCU_MBUS_CLK: u64 = 0x0540;
const CCU_DMA_BGR: u64 = 0x070c;
const CCU_UART_BGR: u64 = 0x090C;
const CCU_RISCV_CLK: u64 = 0x0d00;

pub unsafe fn init_uart() {
    unsafe {
        Reg32::read(CCU_BASE + CCU_UART_BGR)
            .set_field::<0, 1>(1) // UART0 gating enable
            .set_field::<16, 1>(1) // UART0 reset deassert
            .write();
    }
}

pub unsafe fn init_clocks() {
    crate::uart::printf!("initializing clocks\r\n");

    unsafe {
        init_cpu();
        init_peri_pll();
        init_ahb();
        init_apb();
        init_dma();
        init_mbus();
    }

    crate::uart::printf!("clocks initialized\r\n");
}

unsafe fn init_cpu() {
    unsafe {
        crate::uart::printf!("initializing CPU/AXI clocks\r\n");

        // Temporarily reparent RISC core clock to 24MHz HOSC while we're setting up the PLL
        Reg32::zero(CCU_BASE + CCU_RISCV_CLK)
            .set_field::<24, 2>(0) // RISC-V_CLK_SEL = HOSC (0)
            .set_field::<8, 2>(3) // RISC-V_AXI_DIV_CFG (Factor N) = 3
            .set_field::<0, 1>(1) // RISC-V_DIV_CFG (Factor M) = 1
            .write();

        udelay(1);

        // Disable gating
        Reg32::read(CCU_BASE + CCU_PLL_CPU_CTRL)
            .set_field::<27, 1>(0) // PLL_OUTPUT_GATE = 0
            .write();
        Reg32::read(CCU_BASE + CCU_PLL_CPU_CTRL)
            .set_field::<30, 1>(1) // PLL_LDO_EN = 1
            .write();

        udelay(5);

        // PLL freq = 24MHz * N / M  = 24 * 42 / 1 = 1008 Mhz ~ 1GHz
        Reg32::read(CCU_BASE + CCU_PLL_CPU_CTRL)
            .set_field::<8, 8>(41) // PLL_N = 41
            .set_field::<0, 2>(0) // PLL_M = 0
            .write();

        // Enable PLL lock
        Reg32::read(CCU_BASE + CCU_PLL_CPU_CTRL)
            .set_field::<29, 1>(1)
            .write();

        // Enable PLL
        Reg32::read(CCU_BASE + CCU_PLL_CPU_CTRL)
            .set_field::<31, 1>(1)
            .write();

        // Wait for PLL lock to become stable
        Reg32::read(CCU_BASE + CCU_PLL_CPU_CTRL).wait_bit::<28>(true);
        udelay(20);

        // Disable gating
        Reg32::read(CCU_BASE + CCU_PLL_CPU_CTRL)
            .set_field::<27, 1>(1)
            .write();

        // Disable PLL lock
        Reg32::read(CCU_BASE + CCU_PLL_CPU_CTRL)
            .set_field::<29, 1>(0)
            .write();

        udelay(1);

        // - Set CPUX clock source to PLL_CPU
        // - Set CPUX AXI clock to 504 MHz (PLL_CPU / M)
        // - Set CPUX APB clock to 504 MHz (PLL_CPU / N)
        Reg32::read(CCU_BASE + CCU_CPU_AXI_CFG)
            .set_field::<0, 2>(1) // CPU_DIV1 (factor M - 1) = 1
            .set_field::<8, 2>(1) // CPU_DIV2 (factor N - 1) = 1
            .set_field::<24, 3>(3) // CPU_CLK_SEL = PLL_CPU  (3)
            .write();

        udelay(1);

        // - Reparent RISCV core clock to CPU PLL
        // - RISCV core clock freq = PLL_CPU / M = 1008 MHz ~ 1 GHz
        // - RISCV AXI freq = PLL_CPU / N = 504 MHz
        Reg32::read(CCU_BASE + CCU_RISCV_CLK)
            .set_field::<0, 4>(0) // RISC-V_DIV_CFG (factor M - 1) = 0
            .set_field::<8, 2>(1) // RISC-V_AXI_DIV_CFG (factor N - 1) = 1
            .set_field::<24, 3>(5) // RISC-V_CLK_SEL = PLL_CPU
            .write();

        udelay(1);

        let pll_clk = {
            let reg = Reg32::read(CCU_BASE + CCU_PLL_CPU_CTRL);

            24_000_000 * (reg.field::<8, 8>() + 1) as u64 / (reg.field::<0, 2>() as u64 + 1)
        };

        let (cpux_clk, cpux_axi_clk, cpux_apb_clk) = {
            let reg = Reg32::read(CCU_BASE + CCU_CPU_AXI_CFG);

            let cpux_clk = pll_clk;

            (
                cpux_clk,
                cpux_clk * (reg.field::<0, 2>() + 1) as u64,
                cpux_clk * (reg.field::<8, 2>() + 1) as u64,
            )
        };

        let (riscv_clk, riscv_axi_clk) = {
            let reg = Reg32::read(CCU_BASE + CCU_RISCV_CLK);

            let riscv_clk = pll_clk / (reg.field::<0, 4>() + 1) as u64;
            let riscv_axi_clk = pll_clk / (reg.field::<8, 2>() + 1) as u64;

            (riscv_clk, riscv_axi_clk)
        };

        crate::uart::printf!("CPU PLL: %dHz\r\n", pll_clk);
        crate::uart::printf!(
            "CPU: %dMHz, CPU AXI: %dMHz, CPU APB: %dHz\r\n",
            cpux_clk,
            cpux_axi_clk,
            cpux_apb_clk
        );
        crate::uart::printf!(
            "RISCV core: %dHz, RISCV AXI: %dHz\r\n",
            riscv_clk,
            riscv_axi_clk
        );
    }
}

unsafe fn init_peri_pll() {
    unsafe {
        crate::uart::printf!("initializing PERI PLL\r\n");

        let reg = Reg32::read(CCU_BASE + CCU_PLL_PERI_CTRL);

        let n = reg.field::<8, 8>() as u64 + 1; // PLL_N
        let m = reg.field::<1, 1>() as u64 + 1; // PLL_INPUT_DIV2
        let p0 = reg.field::<16, 3>() as u64 + 1; // PLL_P0
        let p1 = reg.field::<20, 3>() as u64 + 1; // PLL_P1

        let hosc = 24_000_000;
        let peri_2x_clk = hosc * n / m / p0;
        let peri_1x_clk = hosc * n / m / p0 / 2;
        let peri_800m_clk = hosc * n / m / p1;

        crate::uart::printf!("PERI PLL (2x): %dHz\r\n", peri_2x_clk);
        crate::uart::printf!("PERI PLL (1x): %dHz\r\n", peri_1x_clk);
        crate::uart::printf!("PERI PLL (800m): %dHz\r\n", peri_800m_clk);
    }
}

unsafe fn init_ahb() {
    unsafe {
        // AHB clock = PLL_PERI(1X) / M / N = 200Mhz

        Reg32::zero(CCU_BASE + CCU_PSI_CLK)
            .set_field::<0, 2>(2) // FACTOR_M = 0
            .set_field::<8, 2>(0) // FACTOR_N = 0
            .write();

        Reg32::read(CCU_BASE + CCU_PSI_CLK)
            .set_field::<24, 2>(3) // CLK_SRC_SEL = PLL_PERI(1X)
            .write();

        udelay(1);

        let psi_ahb_clk = {
            let reg = Reg32::read(CCU_BASE + CCU_PSI_CLK);

            let m = reg.field::<0, 2>() as u64 + 1;
            let n = reg.field::<8, 2>() as u64 + 1;

            600_000_000 / m / n
        };

        crate::uart::printf!("AHB: %dHz", psi_ahb_clk);
    }
}

unsafe fn init_apb() {
    unsafe {
        crate::uart::printf!("initializing APB clock\r\n");

        // APB0 CLK = PLL_PERI (1x) / M / N = 100Mhz

        Reg32::zero(CCU_BASE + CCU_APB0_CLK)
            .set_field::<0, 5>(2) // FACTOR_M = 2
            .set_field::<8, 2>(1) // FACTOR_N = 1
            .write();

        Reg32::read(CCU_BASE + CCU_APB0_CLK)
            .set_field::<24, 2>(3) // CLK_SRC_SEL = PLL_PERI(1X)
            .write();

        udelay(1);

        let apb0_clk = {
            let reg = Reg32::read(CCU_BASE + CCU_APB0_CLK);

            let m = reg.field::<0, 5>() as u64 + 1;
            let n = reg.field::<8, 2>() as u64 + 1;

            200_000_000 / m / n
        };

        crate::uart::printf!("AHB: %dHz", apb0_clk);
    }
}

unsafe fn init_dma() {
    unsafe {
        crate::uart::printf!("initializing DMA clock\r\n");

        // DMA reset
        Reg32::read(CCU_BASE + CCU_DMA_BGR)
            .set_field::<16, 1>(1) // DMA_RST = 1 (de-assert)
            .write();

        udelay(20);

        // DMA Gating clock pass
        Reg32::read(CCU_BASE + CCU_DMA_BGR)
            .set_field::<0, 1>(1) // DMA_GATING = 1 (Pass)
            .write();
    }
}

unsafe fn init_mbus() {
    unsafe {
        crate::uart::printf!("initializing MBUS clock\r\n");

        // Reset MBUS domain
        Reg32::read(CCU_BASE + CCU_MBUS_CLK)
            .set_field::<30, 1>(1) // MBUS_RST = 1 (de-assert)
            .write();

        udelay(1);
    }
}
