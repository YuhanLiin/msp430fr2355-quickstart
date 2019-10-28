use msp430fr2355 as pac;
use pac::cs::csctl1::DCORSEL_A;
use pac::cs::csctl4::{SELA_A, SELMS_A};

pub const REFOCLK: u16 = 32768;
pub const VLOCLK: u16 = 10000;
pub const DCOCLK_MAX: u32 = REFOCLK as u32 * 768;

const MCLK_DIV_EXP: u8 = 7;
const MAX_DCO_MUL_EXP: u8 = 10;

pub trait CsExt {
    fn constrain(self) -> ClocksConfig<Undefined>;
}

impl CsExt for pac::CS {
    fn constrain(self) -> ClocksConfig<Undefined> {
        // These are the microcontroller default settings
        ClocksConfig {
            periph: self,
            mode: Undefined,
            mclk_freq: REFOCLK as u32,
            mclk_div: 0,
            mclk_sel: MclkSel::Refoclk,
            aclk_sel: AclkSel::Refoclk,
        }
    }
}

enum MclkSel {
    Refoclk,
    Vloclk,
    Dcoclk { multiplier: u16, range: DCORSEL_A },
}

impl MclkSel {
    fn selms(&self) -> SELMS_A {
        match self {
            MclkSel::Refoclk => SELMS_A::REFOCLK,
            MclkSel::Vloclk => SELMS_A::VLOCLK,
            MclkSel::Dcoclk {
                multiplier: _,
                range: _,
            } => SELMS_A::DCOCLKDIV,
        }
    }
}

pub struct ClocksConfig<MODE> {
    periph: pac::CS,
    mclk_sel: MclkSel,
    mclk_div: u8,
    mclk_freq: u32,
    aclk_sel: AclkSel,
    mode: MODE,
}

macro_rules! mk_clkconf {
    ($conf:expr, $mode:expr) => {
        ClocksConfig {
            periph: $conf.periph,
            mclk_sel: $conf.mclk_sel,
            mclk_div: $conf.mclk_div,
            mclk_freq: $conf.mclk_freq,
            aclk_sel: $conf.aclk_sel,
            mode: $mode,
        }
    };
}

// Makes sure MCLK is only defined before SMCLK
pub struct Undefined;
pub struct MclkDefined;
pub struct SmclkDefined(u8);
pub struct SmclkDisabled;

pub trait SmclkState {
    fn div(&self) -> Option<u8>;
}

impl SmclkState for SmclkDefined {
    fn div(&self) -> Option<u8> {
        Some(self.0)
    }
}

impl SmclkState for SmclkDisabled {
    fn div(&self) -> Option<u8> {
        None
    }
}

#[derive(Clone, Copy)]
enum AclkSel {
    Vloclk,
    Refoclk,
}

impl AclkSel {
    fn to_sela(self) -> SELA_A {
        match self {
            AclkSel::Vloclk => SELA_A::VLOCLK,
            AclkSel::Refoclk => SELA_A::REFOCLK,
        }
    }

    fn freq(self) -> u16 {
        match self {
            AclkSel::Vloclk => VLOCLK as u16,
            AclkSel::Refoclk => REFOCLK as u16,
        }
    }
}

#[derive(Debug)]
pub enum ClockFreqError {
    TooHigh,
    TooLow,
}

impl<MODE> ClocksConfig<MODE> {
    fn match_clk_spec(
        hz: u16,
        max_freq: u16,
        max_div_exp: u8,
    ) -> Result<(u16, u8), ClockFreqError> {
        if hz > max_freq {
            Err(ClockFreqError::TooHigh)
        } else if hz < max_freq >> max_div_exp {
            Err(ClockFreqError::TooLow)
        } else {
            for div in (0..max_div_exp + 1).rev() {
                let current_freq = max_freq >> div;
                if hz <= current_freq {
                    let prev_freq = max_freq >> (div + 1);
                    let cur_diff = current_freq - hz;
                    let prev_diff = hz - prev_freq;

                    let clk_div = if cur_diff < prev_diff { div } else { div + 1 } as u8;
                    let clk_freq = max_freq >> clk_div;
                    return Ok((clk_freq, clk_div));
                }
            }
            return Ok((max_freq >> max_div_exp, max_div_exp));
        }
    }

    pub const fn aclk_refoclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Refoclk;
        self
    }

    pub const fn aclk_vloclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Vloclk;
        self
    }
}

fn determine_dco_range(hz: u32) -> DCORSEL_A {
    let fll_ref = REFOCLK as u32;
    if hz < fll_ref * 32 {
        DCORSEL_A::DCORSEL_0
    } else if hz < fll_ref * 64 {
        DCORSEL_A::DCORSEL_1
    } else if hz < fll_ref * 128 {
        DCORSEL_A::DCORSEL_2
    } else if hz < fll_ref * 256 {
        DCORSEL_A::DCORSEL_3
    } else if hz < fll_ref * 384 {
        DCORSEL_A::DCORSEL_4
    } else if hz < fll_ref * 512 {
        DCORSEL_A::DCORSEL_5
    } else if hz < fll_ref * 640 {
        DCORSEL_A::DCORSEL_6
    } else {
        DCORSEL_A::DCORSEL_7
    }
}

impl ClocksConfig<Undefined> {
    pub fn mclk_refoclk(self, hz: u16) -> Result<ClocksConfig<MclkDefined>, ClockFreqError> {
        let (mclk_freq, mclk_div) = Self::match_clk_spec(hz, REFOCLK, MCLK_DIV_EXP)?;
        let mclk_sel = MclkSel::Refoclk;
        Ok(ClocksConfig {
            mclk_div,
            mclk_freq: mclk_freq as u32,
            mclk_sel,
            ..mk_clkconf!(self, MclkDefined)
        })
    }

    pub fn mclk_vloclk(self, hz: u16) -> Result<ClocksConfig<MclkDefined>, ClockFreqError> {
        let (mclk_freq, mclk_div) = Self::match_clk_spec(hz, VLOCLK, MCLK_DIV_EXP)?;
        let mclk_sel = MclkSel::Vloclk;
        Ok(ClocksConfig {
            mclk_div,
            mclk_freq: mclk_freq as u32,
            mclk_sel,
            ..mk_clkconf!(self, MclkDefined)
        })
    }

    pub fn mclk_dcoclk(self, hz: u32) -> Result<ClocksConfig<MclkDefined>, ClockFreqError> {
        let fll_ref = REFOCLK as u32;
        if hz < fll_ref {
            Err(ClockFreqError::TooLow)
        } else if hz > DCOCLK_MAX {
            Err(ClockFreqError::TooHigh)
        } else {
            let mut multiplier = hz / fll_ref;
            if hz % fll_ref > fll_ref / 2 {
                multiplier += 1;
            }

            let mclk_freq = multiplier * fll_ref;
            Ok(ClocksConfig {
                mclk_div: 0,
                mclk_freq,
                mclk_sel: MclkSel::Dcoclk {
                    multiplier: multiplier as u16,
                    range: determine_dco_range(hz),
                },
                ..mk_clkconf!(self, MclkDefined)
            })
        }
    }

    pub const fn mclk_default(self) -> ClocksConfig<MclkDefined> {
        mk_clkconf!(self, MclkDefined)
    }
}

impl ClocksConfig<MclkDefined> {
    pub const fn smclk_divide_1(self) -> ClocksConfig<SmclkDefined> {
        mk_clkconf!(self, SmclkDefined(0))
    }

    pub const fn smclk_divide_2(self) -> ClocksConfig<SmclkDefined> {
        mk_clkconf!(self, SmclkDefined(1))
    }

    pub const fn smclk_divide_4(self) -> ClocksConfig<SmclkDefined> {
        mk_clkconf!(self, SmclkDefined(2))
    }

    pub const fn smclk_divide_8(self) -> ClocksConfig<SmclkDefined> {
        mk_clkconf!(self, SmclkDefined(3))
    }

    pub const fn smclk_off(self) -> ClocksConfig<SmclkDisabled> {
        mk_clkconf!(self, SmclkDisabled)
    }
}

impl<SMCLK: SmclkState> ClocksConfig<SMCLK> {
    fn configure_periph(&self) {
        if let MclkSel::Dcoclk { multiplier, range } = self.mclk_sel {
            // Turn off FLL if it were possible
            self.periph.csctl3.write(|w| w.selref().refoclk());
            self.periph.csctl0.write(|w| unsafe { w.bits(0) });
            self.periph.csctl1.write(|w| w.dcorsel().variant(range));
            self.periph
                .csctl2
                .write(|w| unsafe { w.flln().bits(multiplier) }.flld()._1());
            // Turn on FLL if it were possible

            msp430::asm::nop();
            msp430::asm::nop();
            msp430::asm::nop();
            while !self.periph.csctl7.read().fllunlock().is_fllunlock_0() {}
        }

        self.periph.csctl4.write(|w| {
            w.sela()
                .variant(self.aclk_sel.to_sela())
                .selms()
                .variant(self.mclk_sel.selms())
        });

        self.periph.csctl5.write(|w| {
            let w = w.vloautooff().set_bit().divm().bits(self.mclk_div);
            match self.mode.div() {
                Some(div) => w.divs().bits(div),
                None => w.smclkoff().set_bit(),
            }
        });
    }
}

impl ClocksConfig<SmclkDefined> {
    pub fn freeze(self) -> (Mclk, Smclk, Aclk) {
        self.configure_periph();
        (
            Mclk(self.mclk_freq),
            Smclk(self.mclk_freq >> self.mode.0),
            Aclk(self.aclk_sel.freq()),
        )
    }
}

impl ClocksConfig<SmclkDisabled> {
    pub fn freeze(self) -> (Mclk, Aclk) {
        self.configure_periph();
        (Mclk(self.mclk_freq), Aclk(self.aclk_sel.freq()))
    }
}

pub struct Mclk(u32);
pub struct Smclk(u32);
pub struct Aclk(u16);

pub trait Clock {
    type Freq;

    fn freq(&self) -> Self::Freq;
}

impl Clock for Mclk {
    type Freq = u32;

    fn freq(&self) -> u32 {
        self.0
    }
}

impl Clock for Smclk {
    type Freq = u32;

    fn freq(&self) -> u32 {
        self.0
    }
}

impl Clock for Aclk {
    type Freq = u16;

    fn freq(&self) -> u16 {
        self.0
    }
}
