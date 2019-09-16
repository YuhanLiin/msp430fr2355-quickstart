#![no_std]

extern crate msp430;
extern crate msp430fr2355;
extern crate panic_msp430;

use core::marker::PhantomData;
use msp430::{asm, interrupt};
use msp430fr2355 as pac;

pub trait CsExt {
    fn constrain(self) -> ClocksConfig;
    fn default_clocks(self) -> Clocks;
}

impl CsExt for pac::CS {
    fn constrain(self) -> ClocksConfig {
        // These are the microcontroller default settings
        ClocksConfig {
            periph: self,
            mclk_freq: MclkFreq::Hz1M,
            mclk_div: MclkDiv::D1,
            smclk_div: Some(SmclkDiv::D1),
            aclk_freq: AclkFreq::Hz32K,
        }
    }

    // Skip the actual configuration and freeze clocks using default settings
    fn default_clocks(self) -> Clocks {
        self.constrain().to_clocks()
    }
}

pub struct ClocksConfig {
    periph: pac::CS,
    mclk_freq: MclkFreq,
    mclk_div: MclkDiv,
    smclk_div: Option<SmclkDiv>,
    aclk_freq: AclkFreq,
}

#[derive(Clone, Copy)]
pub enum MclkFreq {
    Hz1M,
    Hz32K,
    Hz10K,
}

impl Into<u32> for MclkFreq {
    fn into(self) -> u32 {
        match self {
            MclkFreq::Hz1M => 1_000_000,
            MclkFreq::Hz32K => 32768,
            MclkFreq::Hz10K => 10000,
        }
    }
}

#[derive(Clone, Copy)]
pub enum MclkDiv {
    D1 = 0,
    D2 = 1,
    D4 = 2,
    D8 = 3,
    D16 = 4,
    D32 = 5,
    D64 = 6,
    D128 = 7,
}

#[derive(Clone, Copy)]
pub enum SmclkDiv {
    D1 = 0,
    D2 = 1,
    D4 = 2,
    D8 = 3,
}

#[derive(Clone, Copy)]
pub enum AclkFreq {
    Hz32K,
    Hz10K,
}

impl Into<u32> for AclkFreq {
    fn into(self) -> u32 {
        match self {
            AclkFreq::Hz32K => 32768,
            AclkFreq::Hz10K => 10000,
        }
    }
}

impl ClocksConfig {
    pub fn mclk_freq(mut self, freq: MclkFreq) -> Self {
        self.mclk_freq = freq;
        self
    }

    pub fn mclk_div(mut self, div: MclkDiv) -> Self {
        self.mclk_div = div;
        self
    }

    pub fn smclk_div(mut self, div: SmclkDiv) -> Self {
        self.smclk_div = Some(div);
        self
    }

    pub fn smclk_off(mut self) -> Self {
        self.smclk_div = None;
        self
    }

    pub fn aclk_freq(mut self, freq: AclkFreq) -> Self {
        self.aclk_freq = freq;
        self
    }

    pub(crate) fn to_clocks(self) -> Clocks {
        let mclk: u32 = self.mclk_freq.into();
        let mclk_freq = mclk >> (self.mclk_div as u8);
        let smclk_freq = self.smclk_div.map(|sdiv| mclk_freq >> (sdiv as u8));
        let aclk_freq = self.aclk_freq.into();

        Clocks {
            periph: self.periph,
            mclk_freq,
            smclk_freq,
            aclk_freq,
        }
    }

    pub fn freeze(self) -> Clocks {
        use pac::cs::csctl4::SELA_A;
        use pac::cs::csctl4::SELMS_A;

        let mclk_freq = match self.mclk_freq {
            MclkFreq::Hz1M => SELMS_A::DCOCLKDIV,
            MclkFreq::Hz32K => SELMS_A::REFOCLK,
            MclkFreq::Hz10K => SELMS_A::VLOCLK,
        };
        let aclk_freq = match self.aclk_freq {
            AclkFreq::Hz32K => SELA_A::REFOCLK,
            AclkFreq::Hz10K => SELA_A::VLOCLK,
        };
        let mclk_div = self.mclk_div as u8;
        let smclk_div = self.smclk_div.map(|d| d as u8);

        self.periph
            .csctl4
            .write(|w| w.sela().variant(aclk_freq).selms().variant(mclk_freq));

        self.periph.csctl5.write(|w| {
            let w = w.vloautooff().set_bit().divm().bits(mclk_div);
            match smclk_div {
                Some(sdiv) => w.divs().bits(sdiv),
                None => w.smclkoff().set_bit(),
            }
        });

        self.to_clocks()
    }
}

pub struct Clocks {
    periph: pac::CS,
    mclk_freq: u32,
    smclk_freq: Option<u32>,
    aclk_freq: u32,
}

impl Clocks {
    pub fn mclk(&self) -> &u32 {
        &self.mclk_freq
    }

    pub fn smclk(&self) -> &Option<u32> {
        &self.smclk_freq
    }

    pub fn aclk(&self) -> &u32 {
        &self.aclk_freq
    }
}

use pac::wdt_a::wdtctl::{WDTIS_A, WDTSSEL_A};

pub trait WdtExt {
    fn constrain(self) -> Wdt<Watchdog>;
}

impl WdtExt for pac::WDT_A {
    fn constrain(self) -> Wdt<Watchdog> {
        // Disable first
        self.wdtctl
            .write(|w| unsafe { w.wdtpw().bits(0x5A) }.wdthold().hold());
        Wdt {
            _mode: PhantomData,
            periph: self,
            periods: WdtClkPeriods::P2G,
        }
    }
}

pub struct Wdt<MODE> {
    _mode: PhantomData<MODE>,
    periph: pac::WDT_A,
    periods: WdtClkPeriods,
}

pub struct Watchdog;
pub struct Interval;

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

    pub fn set_aclk(self, _clks: &Clocks) -> Self {
        self.set_clk(WDTSSEL_A::ACLK)
    }

    pub fn set_vloclk(self, _clks: &Clocks) -> Self {
        self.set_clk(WDTSSEL_A::VLOCLK)
    }

    pub fn set_smclk(self, clks: &Clocks) -> Result<Self, ()> {
        clks.smclk()
            .ok_or(())
            .map(|_| self.set_clk(WDTSSEL_A::SMCLK))
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
                .variant(periods.into())
        });
    }

    // Don't call this unless type state is also changing
    fn change_mode(&mut self, _mode: bool) {
        self.periph.wdtctl.modify(|r, w| {
            unsafe { w.bits(r.bits()).wdtpw().bits(PASSWORD) }
                // Pause timer when switching modes
                .wdthold()
                .hold()
                .wdttmsel()
                .set_bit()
        });
    }

    pub fn interval_in_clk_periods(&self) -> u32 {
        1 << (self.periods as u8)
    }
}

impl Wdt<Watchdog> {
    pub fn to_interval(mut self) -> Wdt<Interval> {
        self.change_mode(true);
        Wdt {
            _mode: PhantomData,
            periph: self.periph,
            periods: self.periods,
        }
    }
}

impl Wdt<Interval> {
    pub fn to_interval(mut self) -> Wdt<Watchdog> {
        self.change_mode(false);
        Wdt {
            _mode: PhantomData,
            periph: self.periph,
            periods: self.periods,
        }
    }
}

#[derive(Clone, Copy)]
pub enum WdtClkPeriods {
    P64 = 6,
    P512 = 9,
    P8192 = 13,
    P32K = 15,
    P512K = 19,
    P8192K = 23,
    P128M = 27,
    P2G = 31,
}

impl Into<WDTIS_A> for WdtClkPeriods {
    fn into(self) -> WDTIS_A {
        match self {
            WdtClkPeriods::P64 => WDTIS_A::_64,
            WdtClkPeriods::P512 => WDTIS_A::_512,
            WdtClkPeriods::P8192 => WDTIS_A::_8192,
            WdtClkPeriods::P32K => WDTIS_A::_32K,
            WdtClkPeriods::P512K => WDTIS_A::_512K,
            WdtClkPeriods::P8192K => WDTIS_A::_8192K,
            WdtClkPeriods::P128M => WDTIS_A::_128M,
            WdtClkPeriods::P2G => WDTIS_A::_2G,
        }
    }
}

fn main() {
    let periph = pac::Peripherals::take().unwrap();

    let clocks = periph
        .CS
        .constrain()
        .mclk_freq(MclkFreq::Hz1M)
        .mclk_div(MclkDiv::D2)
        .smclk_off()
        .aclk_freq(AclkFreq::Hz10K)
        .freeze();

    let mut wdt = periph.WDT_A.constrain().set_aclk(&clocks);
    wdt.start(WdtClkPeriods::P512);
}
