use crate::clocks::{Aclk, Smclk};
use msp430fr2355 as pac;
use pac::tb0::tb0ctl::TBSSEL_A;
use pac::TB0;

pub struct TimerConfig {
    periph: TB0,
    clk_src: TBSSEL_A,
    div: u8,
    div_ex: u8,
}

impl TimerConfig {
    pub fn use_aclk(mut self, _clk: &Aclk) -> Self {
        self.clk_src = TBSSEL_A::ACLK;
        self
    }

    pub fn use_smclk(mut self, _clk: &Smclk) -> Self {
        self.clk_src = TBSSEL_A::SMCLK;
        self
    }

    pub fn use_inclk(mut self) -> Self {
        self.clk_src = TBSSEL_A::INCLK;
        self
    }

    pub fn use_tbclk(mut self) -> Self {
        self.clk_src = TBSSEL_A::TBCLK;
        self
    }

    pub fn set_div(mut self, div: TimerDiv) -> Self {
        self.div = div as u8;
        self
    }

    pub fn set_div_ex(mut self, div_ex: TimerDivEx) -> Self {
        self.div_ex = div_ex as u8;
        self
    }

    fn write_regs(&self) {
        self.periph.tb0ctl.write(|w| w.tbclr().set_bit());
        self.periph.tb0ex0.write(|w| w.tbidex().bits(self.div_ex));
        self.periph
            .tb0ctl
            .write(|w| w.tbssel().variant(self.clk_src).id().bits(self.div));
    }

    pub fn to_periodic(self) -> TimerParts {
        self.write_regs();

        TimerParts {
            timer: Timer(()),
            sub_timer1: SubTimer1(()),
            sub_timer2: SubTimer2(()),
        }
    }

    pub fn to_pwm(self) -> Pwms {
        self.write_regs();
        // out0 set to toggle, acts as PWM with 50% duty cycle and double the nominal period
        self.periph.tb0cctl1.write(|w| w.outmod().bits(0b100));
        // out1 and out2 set to reset/set acts as normal PWM
        self.periph.tb0cctl1.write(|w| w.outmod().bits(0b111));
        self.periph.tb0cctl2.write(|w| w.outmod().bits(0b111));

        Pwms {
            pwm1: Pwm1(()),
            pwm2: Pwm2(()),
        }
    }

    pub fn config_capture(self) -> CaptureConfig {
        CaptureConfig {
            timer_config: self,
            capture0: CapChannelConfig {
                cap_mode: CaptureMode::NoCapture,
                select: CaptureSelect::Gnd,
            },
            capture1: CapChannelConfig {
                cap_mode: CaptureMode::NoCapture,
                select: CaptureSelect::Gnd,
            },
            capture2: CapChannelConfig {
                cap_mode: CaptureMode::NoCapture,
                select: CaptureSelect::Gnd,
            },
        }
    }
}

pub enum TimerDiv {
    _1,
    _2,
    _4,
    _8,
}

pub enum TimerDivEx {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
}

pub trait TimerExt {
    fn constrain(self) -> TimerConfig;
}

impl TimerExt for TB0 {
    fn constrain(self) -> TimerConfig {
        TimerConfig {
            periph: self,
            clk_src: TBSSEL_A::TBCLK,
            div: 0,
            div_ex: 0,
        }
    }
}

pub struct TimerParts {
    pub timer: Timer,
    pub sub_timer1: SubTimer1,
    pub sub_timer2: SubTimer2,
}

pub struct Timer(());
pub struct SubTimer1(());
pub struct SubTimer2(());

// Touches tbccr0, tbctl
impl Timer {
    // Calling start multiple times without cancelling leads to unreliable behaviour
    pub fn start(&mut self, ticks: u16) {
        let timer = unsafe { &*TB0::ptr() };
        let tbctl = timer.tb0ctl.read();
        if !tbctl.mc().is_stop() {
            timer
                .tb0ctl
                .write(|w| unsafe { w.bits(tbctl.bits()) }.mc().stop());
        }
        timer.tb0ctl.write(|w| {
            unsafe { w.bits(tbctl.bits()) }
                .tbclr()
                .set_bit()
                .tbifg()
                .clear_bit()
                .mc()
                .up()
        });
        timer.tb0ccr0.write(|w| unsafe { w.bits(ticks) });
    }

    // Always None if called before timer has started
    pub fn wait(&mut self) -> Option<()> {
        let timer = unsafe { &*TB0::ptr() };
        let tbctl = timer.tb0ctl.read();
        if tbctl.tbifg().bit() {
            timer
                .tb0ctl
                .write(|w| unsafe { w.bits(tbctl.bits()) }.tbifg().clear_bit());
            Some(())
        } else {
            None
        }
    }

    pub fn cancel(&mut self) -> Result<(), ()> {
        let timer = unsafe { &*TB0::ptr() };
        let tbctl = timer.tb0ctl.read();
        if tbctl.mc().is_stop() {
            Err(())
        } else {
            timer
                .tb0ctl
                .write(|w| unsafe { w.bits(tbctl.bits()) }.mc().stop());
            Ok(())
        }
    }
}

// Touches tbccr1, tbcctl1
impl SubTimer1 {
    pub fn set_count(&mut self, ticks: u16) {
        let timer = unsafe { &*TB0::ptr() };
        timer.tb0ccr1.write(|w| unsafe { w.bits(ticks) });
        timer
            .tb0cctl1
            .modify(|r, w| unsafe { w.bits(r.bits()) }.ccifg().clear_bit());
    }

    pub fn wait(&mut self) -> Option<()> {
        let timer = unsafe { &*TB0::ptr() };
        let cctl = timer.tb0cctl1.read();
        if cctl.ccifg().bit() {
            timer
                .tb0cctl1
                .write(|w| unsafe { w.bits(cctl.bits()) }.ccifg().clear_bit());
            Some(())
        } else {
            None
        }
    }
}

// Touches tbccr2, tbcctl2
impl SubTimer2 {
    pub fn set_count(&mut self, ticks: u16) {
        let timer = unsafe { &*TB0::ptr() };
        timer.tb0ccr2.write(|w| unsafe { w.bits(ticks) });
        timer
            .tb0cctl2
            .modify(|r, w| unsafe { w.bits(r.bits()) }.ccifg().clear_bit());
    }

    pub fn wait(&mut self) -> Option<()> {
        let timer = unsafe { &*TB0::ptr() };
        let cctl = timer.tb0cctl2.read();
        if cctl.ccifg().bit() {
            timer
                .tb0cctl2
                .write(|w| unsafe { w.bits(cctl.bits()) }.ccifg().clear_bit());
            Some(())
        } else {
            None
        }
    }
}

pub struct Pwms {
    pub pwm1: Pwm1,
    pub pwm2: Pwm2,
}

impl Pwms {
    pub fn set_period(&mut self, ticks: u16) {
        let timer = unsafe { &*TB0::ptr() };
        timer.tb0ccr0.write(|w| unsafe { w.bits(ticks) });
    }

    pub fn enable(&mut self) {
        let timer = unsafe { &*TB0::ptr() };
        timer.tb0ctl.modify(|r, w| {
            unsafe { w.bits(r.bits()) }
                .tbclr()
                .set_bit()
                .tbifg()
                .clear_bit()
                .mc()
                .up()
        });
    }

    pub fn disable(&mut self) {
        let timer = unsafe { &*TB0::ptr() };
        timer
            .tb0ctl
            .modify(|r, w| unsafe { w.bits(r.bits()) }.mc().stop());
    }
}

pub struct Pwm1(());
pub struct Pwm2(());

// If duty > period, output signal stays high
impl Pwm1 {
    pub fn set_duty(&mut self, ticks: u16) {
        let timer = unsafe { &*TB0::ptr() };
        timer.tb0ccr1.write(|w| unsafe { w.bits(ticks) });
    }
}

impl Pwm2 {
    pub fn set_duty(&mut self, ticks: u16) {
        let timer = unsafe { &*TB0::ptr() };
        timer.tb0ccr2.write(|w| unsafe { w.bits(ticks) });
    }
}

pub struct CaptureConfig {
    timer_config: TimerConfig,
    capture0: CapChannelConfig,
    capture1: CapChannelConfig,
    capture2: CapChannelConfig,
}

pub struct CapChannelConfig {
    cap_mode: CaptureMode,
    select: CaptureSelect,
}

#[derive(Clone, Copy)]
pub enum CaptureMode {
    NoCapture,
    Rising,
    Falling,
    Both,
}

#[derive(Clone, Copy)]
pub enum CaptureSelect {
    CapInputA,
    CapInputB,
    Gnd,
    Vcc,
}

impl CaptureConfig {
    pub fn config_chan0(mut self, cap_mode: CaptureMode, select: CaptureSelect) -> Self {
        self.capture0 = CapChannelConfig { cap_mode, select };
        self
    }

    pub fn config_chan1(mut self, cap_mode: CaptureMode, select: CaptureSelect) -> Self {
        self.capture1 = CapChannelConfig { cap_mode, select };
        self
    }

    pub fn config_chan2(mut self, cap_mode: CaptureMode, select: CaptureSelect) -> Self {
        self.capture2 = CapChannelConfig { cap_mode, select };
        self
    }

    pub fn freeze(self) -> Capture {
        self.timer_config.write_regs();
        self.timer_config.periph.tb0cctl0.write(|w| {
            w.cap()
                .capture()
                .scs()
                .sync()
                .cm()
                .bits(self.capture0.cap_mode as u8)
                .ccis()
                .bits(self.capture0.select as u8)
        });

        self.timer_config.periph.tb0cctl1.write(|w| {
            w.cap()
                .capture()
                .scs()
                .sync()
                .cm()
                .bits(self.capture1.cap_mode as u8)
                .ccis()
                .bits(self.capture1.select as u8)
        });

        self.timer_config.periph.tb0cctl2.write(|w| {
            w.cap()
                .capture()
                .scs()
                .sync()
                .cm()
                .bits(self.capture2.cap_mode as u8)
                .ccis()
                .bits(self.capture2.select as u8)
        });

        self.timer_config.periph.tb0ctl.modify(|r, w| {
            unsafe { w.bits(r.bits()) }
                .tbclr()
                .set_bit()
                .mc()
                .continuous()
        });

        Capture {
            capture0: CaptureChannnel0(()),
            capture1: CaptureChannnel1(()),
            capture2: CaptureChannnel2(()),
        }
    }
}

pub struct Capture {
    pub capture0: CaptureChannnel0,
    pub capture1: CaptureChannnel1,
    pub capture2: CaptureChannnel2,
}

pub struct CaptureChannnel0(());
pub struct CaptureChannnel1(());
pub struct CaptureChannnel2(());

impl CaptureChannnel1 {
    fn clear(&mut self, cctl: u16) {
        let timer = unsafe { &*TB0::ptr() };
        timer.tb0cctl1.write(|w| {
            unsafe { w.bits(cctl) }
                .ccifg()
                .clear_bit()
                .cov()
                .clear_bit()
        });
    }

    pub fn capture(&mut self) -> Result<Option<u16>, u16> {
        let timer = unsafe { &*TB0::ptr() };
        let cctl = timer.tb0cctl1.read();
        if cctl.cov().bit() {
            self.clear(cctl.bits());
            Err(timer.tb0ccr1.read().bits())
        } else if cctl.ccifg().bit() {
            let val = timer.tb0ccr1.read().bits();
            // Read cctl again to prevent overrun races
            if timer.tb0cctl1.read().cov().bit() {
                self.clear(cctl.bits());
                Err(val)
            } else {
                self.clear(cctl.bits());
                Ok(Some(val))
            }
        } else {
            Ok(None)
        }
    }
}
