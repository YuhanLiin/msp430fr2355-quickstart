use core::marker::PhantomData;
use msp430fr2355 as pac;

pub trait PmmExt {
    fn freeze(self) -> Pmm;
}

pub struct Pmm;

impl PmmExt for pac::PMM {
    fn freeze(self) -> Pmm {
        self.pm5ctl0.write(|w| w.locklpm5().locklpm5_0());
        Pmm
    }
}

pub trait GpioExt {
    type Gpio;

    fn constrain(self) -> Self::Gpio;
}

pub struct Output;
pub struct Input<PULL, INTR> {
    _pull: PhantomData<PULL>,
    _intr: PhantomData<INTR>,
}

pub struct Pulled;
pub struct Floating;
pub struct Unknown;
pub struct Enabled;
pub struct Disabled;

pub struct Locked;
pub struct Unlocked;

pub trait ConvertToInput {}
impl ConvertToInput for Output {}
impl ConvertToInput for Unknown {}

pub trait ConvertToOutput {}
impl<PULL> ConvertToOutput for Input<PULL, Disabled> {}
impl ConvertToOutput for Unknown {}

pub trait Known {}
impl Known for Floating {}
impl Known for Pulled {}

pub struct P1<DIR, LOCK> {
    periph: pac::P1,
    _dir: PhantomData<DIR>,
    _lock: PhantomData<LOCK>,
}

macro_rules! make_periph {
    ($Px:ident, $periph:expr) => {
        $Px {
            periph: $periph,
            _dir: PhantomData,
            _lock: PhantomData,
        }
    };

    ($Px:ident) => {
        $Px {
            _dir: PhantomData,
            _lock: PhantomData,
        }
    };
}

impl GpioExt for pac::P1 {
    type Gpio = P1<Unknown, Locked>;

    fn constrain(self) -> Self::Gpio {
        make_periph!(P1, self)
    }
}

impl<PULL, LOCK> P1<Input<PULL, Disabled>, LOCK> {
    pub fn pulldown(self) -> P1<Input<Pulled, Disabled>, LOCK> {
        self.periph.p1out.write(|w| unsafe { w.bits(0x00) });
        self.periph.p1ren.write(|w| unsafe { w.bits(0xFF) });
        make_periph!(P1, self.periph)
    }

    pub fn pullup(self) -> P1<Input<Pulled, Disabled>, LOCK> {
        self.periph.p1out.write(|w| unsafe { w.bits(0xFF) });
        self.periph.p1ren.write(|w| unsafe { w.bits(0xFF) });
        make_periph!(P1, self.periph)
    }

    pub fn float(self) -> P1<Input<Floating, Disabled>, LOCK> {
        self.periph.p1ren.write(|w| unsafe { w.bits(0x00) });
        make_periph!(P1, self.periph)
    }
}

impl<PULL: Known> P1<Input<PULL, Disabled>, Unlocked> {
    pub fn enable_intr_rising_edge(self) -> P1<Input<PULL, Enabled>, Unlocked> {
        self.periph.p1ies.write(|w| unsafe { w.bits(0x00) });
        self.periph.p1ifg.write(|w| unsafe { w.bits(0x00) });
        self.periph.p1ie.write(|w| unsafe { w.bits(0xFF) });
        make_periph!(P1, self.periph)
    }

    pub fn enable_intr_falling_edge(self) -> P1<Input<PULL, Enabled>, Unlocked> {
        self.periph.p1ies.write(|w| unsafe { w.bits(0xFF) });
        self.periph.p1ifg.write(|w| unsafe { w.bits(0x00) });
        self.periph.p1ie.write(|w| unsafe { w.bits(0xFF) });
        make_periph!(P1, self.periph)
    }
}

impl<PULL> P1<Input<PULL, Enabled>, Unlocked> {
    pub fn disable_intr(self) -> P1<Input<PULL, Disabled>, Unlocked> {
        self.periph.p1ie.write(|w| unsafe { w.bits(0x00) });
        make_periph!(P1, self.periph)
    }
}

impl<PULL: Known, INTR> P1<Input<PULL, INTR>, Unlocked> {
    pub fn read(&self) -> u8 {
        self.periph.p1in.read().bits()
    }

    pub fn clear_intr(&mut self) {
        self.periph.p1ifg.write(|w| unsafe { w.bits(0x00) });
    }

    pub fn set_intr(&mut self) {
        self.periph.p1ifg.write(|w| unsafe { w.bits(0xFF) });
    }
}

impl P1<Output, Unlocked> {
    pub fn write(&mut self, val: u8) {
        self.periph.p1out.write(|w| unsafe { w.bits(val) });
    }

    pub fn toggle(&mut self) {
        self.periph.p1out.modify(|r, w| unsafe { w.bits(r.bits()) });
    }
}

impl<DIR: ConvertToInput, LOCK> P1<DIR, LOCK> {
    pub fn to_input(self) -> P1<Input<Unknown, Disabled>, LOCK> {
        self.periph.p1dir.write(|w| unsafe { w.bits(0x00) });
        make_periph!(P1, self.periph)
    }
}

impl<DIR: ConvertToOutput, LOCK> P1<DIR, LOCK> {
    pub fn to_output(self) -> P1<Output, LOCK> {
        self.periph.p1dir.write(|w| unsafe { w.bits(0xFF) });
        make_periph!(P1, self.periph)
    }
}

impl<DIR> P1<DIR, Locked> {
    pub fn unlock(self, _lock: &Pmm) -> P1<DIR, Unlocked> {
        make_periph!(P1, self.periph)
    }
}

impl<DIR, LOCK> P1<DIR, LOCK> {
    pub fn split(self) -> Parts<DIR, LOCK> {
        Parts {
            p1_0: make_periph!(P1_0),
        }
    }

    pub fn join(_parts: Parts<DIR, LOCK>) -> Self {
        let periph = unsafe { pac::Peripherals::steal().P1 };
        make_periph!(P1, periph)
    }
}

pub struct Parts<DIR, LOCK> {
    pub p1_0: P1_0<DIR, LOCK>,
}

pub struct P1_0<DIR, LOCK> {
    _dir: PhantomData<DIR>,
    _lock: PhantomData<LOCK>,
}

impl<PULL: Known, INTR> P1_0<Input<PULL, INTR>, Unlocked> {
    pub fn read(&self) -> bool {
        unsafe { &*pac::P1::ptr() }.p1in.read().bits() & (1 << 0) != 0
    }
}

impl P1_0<Output, Unlocked> {
    pub fn set_bit(&mut self) {
        unsafe { &*pac::P1::ptr() }
            .p1out
            .write(|w| unsafe { w.bits(1 << 0) });
    }

    pub fn clear_bit(&mut self) {
        unsafe { &*pac::P1::ptr() }
            .p1out
            .write(|w| unsafe { w.bits(!(1 << 0)) });
    }
}

impl<DIR> P1_0<DIR, Locked> {
    pub fn unlock(self, _lock: &Pmm) -> P1_0<DIR, Unlocked> {
        make_periph!(P1_0)
    }
}
