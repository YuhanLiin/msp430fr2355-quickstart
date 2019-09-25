use crate::clocks::{Aclk, Smclk};
use core::marker::PhantomData;
use msp430fr2355 as pac;
use pac::wdt_a::wdtctl::WDTSSEL_A;

pub use pac::wdt_a::wdtctl::WDTIS_A as WdtClkPeriods;

pub trait WdtExt {
    fn constrain(self) -> Wdt<WatchdogMode>;
}

impl WdtExt for pac::WDT_A {
    fn constrain(self) -> Wdt<WatchdogMode> {
        // Disable first
        self.wdtctl
            .write(|w| unsafe { w.wdtpw().bits(0x5A) }.wdthold().hold());
        Wdt {
            _mode: PhantomData,
            periph: self,
        }
    }
}

pub struct Wdt<MODE> {
    _mode: PhantomData<MODE>,
    periph: pac::WDT_A,
}

pub struct WatchdogMode;
pub struct IntervalMode;

const PASSWORD: u8 = 0x5A;

impl<MODE> Wdt<MODE> {
    fn set_clk(self, clk_src: WDTSSEL_A) -> Self {
        let bits = self.periph.wdtctl.read().bits();
        // Halt timer first
        self.periph.wdtctl.write(|w| {
            unsafe { w.bits(bits).wdtpw().bits(PASSWORD) }
                .wdthold()
                .hold()
        });
        // Set clock src
        self.periph.wdtctl.write(|w| {
            unsafe { w.bits(bits).wdtpw().bits(PASSWORD) }
                .wdtssel()
                .variant(clk_src)
        });
        self
    }

    pub fn set_aclk(self, _clks: &Aclk) -> Self {
        self.set_clk(WDTSSEL_A::ACLK)
    }

    pub fn set_vloclk(self) -> Self {
        self.set_clk(WDTSSEL_A::VLOCLK)
    }

    pub fn set_smclk(self, _clks: &Smclk) -> Self {
        self.set_clk(WDTSSEL_A::SMCLK)
    }

    pub fn reset(&mut self) {
        self.periph.wdtctl.modify(|r, w| {
            unsafe { w.bits(r.bits()).wdtpw().bits(PASSWORD) }
                .wdtcntcl()
                .set_bit()
        });
    }

    pub fn disable(&mut self) {
        self.periph.wdtctl.modify(|r, w| {
            unsafe { w.bits(r.bits()).wdtpw().bits(PASSWORD) }
                .wdthold()
                .hold()
        });
    }

    pub fn start(&mut self, periods: WdtClkPeriods) {
        self.periph.wdtctl.modify(|r, w| {
            unsafe { w.bits(r.bits()).wdtpw().bits(PASSWORD) }
                // Reset countdown
                .wdtcntcl()
                .set_bit()
                // Unpause timer
                .wdthold()
                .unhold()
                // Set time
                .wdtis()
                .variant(periods)
        });
    }

    // Don't call this unless type state is also changing
    fn change_mode(&mut self, mode: bool) {
        self.periph.wdtctl.modify(|r, w| {
            unsafe { w.bits(r.bits()).wdtpw().bits(PASSWORD) }
                // Pause timer when switching modes
                .wdthold()
                .hold()
                .wdttmsel()
                .bit(mode)
        });
    }
}

impl Wdt<WatchdogMode> {
    pub fn to_interval(mut self) -> Wdt<IntervalMode> {
        unsafe { &*pac::SFR::ptr() }
            .sfrifg1
            .write(|w| w.wdtifg().clear_bit());
        self.change_mode(true);
        Wdt {
            _mode: PhantomData,
            periph: self.periph,
        }
    }
}

impl Wdt<IntervalMode> {
    pub fn to_watchdog(mut self) -> Wdt<WatchdogMode> {
        self.change_mode(false);
        Wdt {
            _mode: PhantomData,
            periph: self.periph,
        }
    }

    pub fn wait_done(&mut self) -> bool {
        let sfr = unsafe { &*pac::SFR::ptr() };
        if sfr.sfrifg1.read().wdtifg().is_wdtifg_1() {
            sfr.sfrifg1.write(|w| w.wdtifg().clear_bit());
            true
        } else {
            false
        }
    }
}
