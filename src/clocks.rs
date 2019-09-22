use core::marker::PhantomData;
use msp430fr2355 as pac;
use pac::cs::csctl1::DCORSEL_A;
use pac::cs::csctl4::{SELA_A, SELMS_A};

const REFOCLK: u32 = 32768;
const VLOCLK: u32 = 10000;
const DCOCLK_MAX: u32 = REFOCLK * 768;

const MCLK_DIV_EXP: u8 = 7;
const SMCLK_DIV_EXP: u8 = 3;
const MAX_DCO_MUL_EXP: u8 = 10;

pub trait CsExt {
    fn constrain(self) -> ClocksConfig<Undefined>;
}

impl CsExt for pac::CS {
    fn constrain(self) -> ClocksConfig<Undefined> {
        // These are the microcontroller default settings
        ClocksConfig {
            periph: self,
            _mode: PhantomData,
            mclk_freq: REFOCLK,
            mclk_div: 0,
            mclk_sel: MclkSel::Refoclk,
            smclk_freq: Some(REFOCLK),
            smclk_div: 0,
            aclk_freq: AclkFreq::Refoclk,
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
    smclk_div: u8,
    smclk_freq: Option<u32>,
    aclk_freq: AclkFreq,
    _mode: PhantomData<MODE>,
}

macro_rules! mk_clkconf {
    ($conf:expr) => {
        ClocksConfig {
            periph: $conf.periph,
            mclk_sel: $conf.mclk_sel,
            mclk_div: $conf.mclk_div,
            mclk_freq: $conf.mclk_freq,
            smclk_div: $conf.smclk_div,
            smclk_freq: $conf.smclk_freq,
            aclk_freq: $conf.aclk_freq,
            _mode: PhantomData,
        }
    };
}

// Makes sure MCLK is only defined before SMCLK
pub struct Undefined;
pub struct MclkDefined;
pub struct SmclkDefined;

#[derive(Clone, Copy)]
enum AclkFreq {
    Vloclk,
    Refoclk,
}

impl AclkFreq {
    fn to_sela(self) -> SELA_A {
        match self {
            AclkFreq::Vloclk => SELA_A::VLOCLK,
            AclkFreq::Refoclk => SELA_A::REFOCLK,
        }
    }

    fn freq(self) -> u16 {
        match self {
            AclkFreq::Vloclk => VLOCLK as u16,
            AclkFreq::Refoclk => REFOCLK as u16,
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
        hz: u32,
        max_freq: u32,
        max_div_exp: u8,
        infallible: bool,
    ) -> Result<(u32, u8), ClockFreqError> {
        if hz > max_freq && !infallible {
            Err(ClockFreqError::TooHigh)
        } else if hz < max_freq >> max_div_exp && !infallible {
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

    pub fn aclk_refoclk(mut self) -> Self {
        self.aclk_freq = AclkFreq::Refoclk;
        self
    }

    pub fn aclk_vloclk(mut self) -> Self {
        self.aclk_freq = AclkFreq::Vloclk;
        self
    }
}

fn determine_dco_range(hz: u32) -> DCORSEL_A {
    if hz < REFOCLK * 32 {
        DCORSEL_A::DCORSEL_0
    } else if hz < REFOCLK * 64 {
        DCORSEL_A::DCORSEL_1
    } else if hz < REFOCLK * 128 {
        DCORSEL_A::DCORSEL_2
    } else if hz < REFOCLK * 256 {
        DCORSEL_A::DCORSEL_3
    } else if hz < REFOCLK * 384 {
        DCORSEL_A::DCORSEL_4
    } else if hz < REFOCLK * 512 {
        DCORSEL_A::DCORSEL_5
    } else if hz < REFOCLK * 640 {
        DCORSEL_A::DCORSEL_6
    } else {
        DCORSEL_A::DCORSEL_7
    }
}

impl ClocksConfig<Undefined> {
    pub fn mclk_refoclk(self, hz: u16) -> Result<ClocksConfig<MclkDefined>, ClockFreqError> {
        let (mclk_freq, mclk_div) = Self::match_clk_spec(hz as u32, REFOCLK, MCLK_DIV_EXP, false)?;
        let mclk_sel = MclkSel::Refoclk;
        Ok(ClocksConfig {
            mclk_div,
            mclk_freq,
            mclk_sel,
            ..mk_clkconf!(self)
        })
    }

    pub fn mclk_vloclk(self, hz: u16) -> Result<ClocksConfig<MclkDefined>, ClockFreqError> {
        let (mclk_freq, mclk_div) = Self::match_clk_spec(hz as u32, VLOCLK, MCLK_DIV_EXP, false)?;
        let mclk_sel = MclkSel::Vloclk;
        Ok(ClocksConfig {
            mclk_div,
            mclk_freq,
            mclk_sel,
            ..mk_clkconf!(self)
        })
    }

    pub fn mclk_dcoclk(self, hz: u32) -> Result<ClocksConfig<MclkDefined>, ClockFreqError> {
        if hz < (REFOCLK >> MCLK_DIV_EXP) as u32 {
            Err(ClockFreqError::TooLow)
        } else if hz > DCOCLK_MAX {
            Err(ClockFreqError::TooHigh)
        } else {
            let (div, hz_err, div_freq) = (0..MCLK_DIV_EXP + 1)
                .filter_map(|div| {
                    let peak_freq = REFOCLK << (MAX_DCO_MUL_EXP - div);
                    let div_freq = (REFOCLK >> div) as u32;
                    if peak_freq < hz || div_freq as u32 > hz {
                        None
                    } else {
                        let hz_err = hz % div_freq;
                        Some((div, hz_err, div_freq))
                    }
                })
                .min_by_key(|(_, hz_err, _)| *hz_err)
                .unwrap();

            let mut multiplier = (hz / div_freq) as u16;
            if hz_err > div_freq / 2 {
                multiplier += 1;
            }
            let mclk_freq = multiplier as u32 * div_freq as u32;
            let range = determine_dco_range(mclk_freq << div);
            let mclk_sel = MclkSel::Dcoclk { range, multiplier };
            let mclk_div = div;
            Ok(ClocksConfig {
                mclk_div,
                mclk_freq,
                mclk_sel,
                ..mk_clkconf!(self)
            })
        }
    }

    pub fn mclk_default(self) -> ClocksConfig<MclkDefined> {
        mk_clkconf!(self)
    }
}

impl ClocksConfig<MclkDefined> {
    pub fn smclk_on(self, hz: u32) -> ClocksConfig<SmclkDefined> {
        let (smclk_freq, smclk_div) =
            Self::match_clk_spec(hz as u32, self.mclk_freq, SMCLK_DIV_EXP, true).unwrap();
        ClocksConfig {
            smclk_div,
            smclk_freq: Some(smclk_freq),
            ..mk_clkconf!(self)
        }
    }

    pub fn smclk_off(self) -> ClocksConfig<SmclkDefined> {
        ClocksConfig {
            smclk_freq: None,
            ..mk_clkconf!(self)
        }
    }

    pub fn smclk_default(self) -> ClocksConfig<SmclkDefined> {
        mk_clkconf!(self)
    }
}

impl ClocksConfig<SmclkDefined> {
    pub fn freeze(self) -> Clocks {
        if let MclkSel::Dcoclk { multiplier, range } = self.mclk_sel {
            // Turn off FLL if it were possible
            self.periph.csctl3.write(|w| w.selref().refoclk());
            self.periph.csctl0.write(|w| unsafe { w.bits(0) });
            self.periph
                .csctl1
                .write(|w| w.dcorsel().variant(range).dismod().set_bit());
            self.periph
                .csctl2
                .write(|w| unsafe { w.flln().bits(multiplier) }.flld()._1());
            // Turn on FLL if it were possible
        }

        self.periph.csctl4.write(|w| {
            w.sela()
                .variant(self.aclk_freq.to_sela())
                .selms()
                .variant(self.mclk_sel.selms())
        });

        self.periph.csctl5.write(|w| {
            let w = w.vloautooff().set_bit().divm().bits(self.mclk_div);
            match self.smclk_freq {
                Some(_) => w.divs().bits(self.smclk_div),
                None => w.smclkoff().set_bit(),
            }
        });

        Clocks {
            mclk_freq: self.mclk_freq,
            smclk_freq: self.smclk_freq,
            aclk_freq: self.aclk_freq.freq(),
        }
    }
}

pub struct Clocks {
    mclk_freq: u32,
    smclk_freq: Option<u32>,
    aclk_freq: u16,
}

impl Clocks {
    pub fn mclk(&self) -> u32 {
        self.mclk_freq
    }

    pub fn smclk(&self) -> Option<u32> {
        self.smclk_freq
    }

    pub fn aclk(&self) -> u16 {
        self.aclk_freq
    }
}
