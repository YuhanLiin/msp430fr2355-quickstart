use crate::clocks::{Aclk, Clock, Smclk};
use msp430fr2355 as pac;

use pac::e_usci_a1::uca1ctlw0::{UC7BIT_A, UCMSB_A, UCSPB_A, UCSSEL_A};
use pac::E_USCI_A1;

enum Parity {
    Even,
    Odd,
    NoParity,
}

pub struct NoBaudConfig;

pub enum BaudConfig {
    Over16 { br: u16, brs: u8, brf: u8 },
    Under16 { br: u16, brs: u8 },
}

pub struct SerialConfig<BAUD> {
    periph: E_USCI_A1,
    bit_order: UCMSB_A,
    bit_cnt: UC7BIT_A,
    stop_bits: UCSPB_A,
    parity: Parity,
    clk_sel: UCSSEL_A,
    baud_config: BAUD,
}

macro_rules! mk_config {
    ($conf:expr, $baud:expr, $sel:expr) => {
        SerialConfig {
            periph: $conf.periph,
            bit_order: $conf.bit_order,
            bit_cnt: $conf.bit_cnt,
            stop_bits: $conf.stop_bits,
            parity: $conf.parity,
            clk_sel: $sel,
            baud_config: $baud,
        }
    };
}

pub trait UsciExt {
    fn constrain(self) -> SerialConfig<NoBaudConfig>;
}

impl UsciExt for E_USCI_A1 {
    fn constrain(self) -> SerialConfig<NoBaudConfig> {
        SerialConfig {
            periph: self,
            bit_order: UCMSB_A::UCMSB_0,
            bit_cnt: UC7BIT_A::_8BIT,
            stop_bits: UCSPB_A::UCSPB_0,
            parity: Parity::NoParity,
            clk_sel: UCSSEL_A::ACLK,
            baud_config: NoBaudConfig,
        }
    }
}

impl<BAUD> SerialConfig<BAUD> {
    pub fn msb_first(mut self) -> Self {
        self.bit_order = UCMSB_A::UCMSB_1;
        self
    }

    pub fn lsb_first(mut self) -> Self {
        self.bit_order = UCMSB_A::UCMSB_0;
        self
    }

    pub fn char_7bits(mut self) -> Self {
        self.bit_cnt = UC7BIT_A::_7BIT;
        self
    }

    pub fn char_8bits(mut self) -> Self {
        self.bit_cnt = UC7BIT_A::_8BIT;
        self
    }

    pub fn stopbits_1(mut self) -> Self {
        self.stop_bits = UCSPB_A::UCSPB_0;
        self
    }

    pub fn stopbits_2(mut self) -> Self {
        self.stop_bits = UCSPB_A::UCSPB_1;
        self
    }

    pub fn parity_none(mut self) -> Self {
        self.parity = Parity::NoParity;
        self
    }

    pub fn parity_even(mut self) -> Self {
        self.parity = Parity::Even;
        self
    }

    pub fn parity_odd(mut self) -> Self {
        self.parity = Parity::Odd;
        self
    }
}

#[derive(Debug)]
pub enum BaudError {
    BpsTooHigh,
    BpsTooLow,
}

impl SerialConfig<NoBaudConfig> {
    pub fn baudrate_aclk(
        self,
        bps: u32,
        aclk: &Aclk,
    ) -> Result<SerialConfig<BaudConfig>, BaudError> {
        let baud_config = Self::calculate_baud_config(aclk.freq() as u32, bps)?;
        Ok(mk_config!(self, baud_config, UCSSEL_A::ACLK))
    }

    pub fn baudrate_smclk(
        self,
        bps: u32,
        smclk: &Smclk,
    ) -> Result<SerialConfig<BaudConfig>, BaudError> {
        let baud_config = Self::calculate_baud_config(smclk.freq(), bps)?;
        Ok(mk_config!(self, baud_config, UCSSEL_A::SMCLK))
    }

    pub fn baudrate_external_uclk(
        self,
        bps: u32,
        clk_freq: u32,
    ) -> Result<SerialConfig<BaudConfig>, BaudError> {
        let baud_config = Self::calculate_baud_config(clk_freq, bps)?;
        Ok(mk_config!(self, baud_config, UCSSEL_A::UCLK))
    }

    fn calculate_baud_config(clk_freq: u32, bps: u32) -> Result<BaudConfig, BaudError> {
        let n = clk_freq / bps;
        if n == 0 {
            Err(BaudError::BpsTooHigh)
        } else if n > 0xFFFF {
            Err(BaudError::BpsTooLow)
        } else {
            let brs = Self::lookup_brs(clk_freq, bps);

            if n >= 16 {
                let div = bps * 16;
                // n / 16, but more precise
                let br = (clk_freq / div) as u16;
                // same as n % 16, but more precise
                let brf = ((clk_freq % div) / bps) as u8;
                Ok(BaudConfig::Over16 { br, brf, brs })
            } else {
                Ok(BaudConfig::Under16 { br: n as u16, brs })
            }
        }
    }

    fn lookup_brs(clk_freq: u32, bps: u32) -> u8 {
        let modulo = clk_freq % bps;

        // Fractional part lookup for the baud rate. Not extremely precise
        if modulo * 19 < bps {
            0x0
        } else if modulo * 14 < bps {
            0x1
        } else if modulo * 12 < bps {
            0x2
        } else if modulo * 10 < bps {
            0x4
        } else if modulo * 8 < bps {
            0x8
        } else if modulo * 7 < bps {
            0x10
        } else if modulo * 6 < bps {
            0x20
        } else if modulo * 5 < bps {
            0x11
        } else if modulo * 4 < bps {
            0x22
        } else if modulo * 3 < bps {
            0x44
        } else if modulo * 11 < bps * 4 {
            0x49
        } else if modulo * 5 < bps * 2 {
            0x4A
        } else if modulo * 7 < bps * 3 {
            0x92
        } else if modulo * 2 < bps {
            0x53
        } else if modulo * 7 < bps * 4 {
            0xAA
        } else if modulo * 13 < bps * 8 {
            0x6B
        } else if modulo * 3 < bps * 2 {
            0xAD
        } else if modulo * 11 < bps * 8 {
            0xD6
        } else if modulo * 4 < bps * 3 {
            0xBB
        } else if modulo * 5 < bps * 4 {
            0xDD
        } else if modulo * 9 < bps * 8 {
            0xEF
        } else {
            0xFD
        }
    }
}

impl SerialConfig<BaudConfig> {
    pub fn freeze(self) -> (Tx, Rx) {
        self.periph.uca1ctlw0().write(|w| w.ucswrst().set_bit());
        match self.baud_config {
            BaudConfig::Over16 { brs, brf, br } => {
                self.periph.uca1brw().write(|w| unsafe { w.bits(br) });
                self.periph
                    .uca1mctlw
                    .write(|w| unsafe { w.ucos16().set_bit().ucbrs().bits(brs).ucbrf().bits(brf) });
            }
            BaudConfig::Under16 { br, brs } => {
                self.periph.uca1brw().write(|w| unsafe { w.bits(br) });
                self.periph
                    .uca1mctlw
                    .write(|w| unsafe { w.ucos16().clear_bit().ucbrs().bits(brs) });
            }
        }

        self.periph.uca1ctlw0().write(|w| {
            w.ucmsb()
                .variant(self.bit_order)
                .uc7bit()
                .variant(self.bit_cnt)
                .ucspb()
                .variant(self.stop_bits)
                .ucssel()
                .variant(self.clk_sel);

            match self.parity {
                Parity::Odd => w.ucpen().set_bit().ucpar().odd(),
                Parity::Even => w.ucpen().set_bit().ucpar().even(),
                Parity::NoParity => w.ucpen().clear_bit(),
            }
        });
        (Tx, Rx)
    }
}

pub struct Tx;
pub struct Rx;

impl Tx {
    pub fn write(&mut self, byte: u8) -> Result<(), ()> {
        let uart = unsafe { &*E_USCI_A1::ptr() };
        if uart.uca1ifg().read().uctxifg().is_uctxifg_0() {
            Err(())
        } else {
            uart.uca1txbuf()
                .write(|w| unsafe { w.uctxbuf().bits(byte) });
            Ok(())
        }
    }
}

impl Rx {
    pub fn read(&self) -> Result<u8, ()> {
        let uart = unsafe { &*E_USCI_A1::ptr() };
        if uart.uca1ifg().read().ucrxifg().is_ucrxifg_0() {
            Err(())
        } else {
            let bits = uart.uca1rxbuf().read().ucrxbuf().bits();
            Ok(bits)
        }
    }
}
