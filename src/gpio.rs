use core::marker::PhantomData;
use msp430fr2355 as pac;

pub trait PmmExt {
    fn freeze(self) -> Pmm;
}

pub struct Pmm(());

impl PmmExt for pac::PMM {
    fn freeze(self) -> Pmm {
        self.pm5ctl0.write(|w| w.locklpm5().locklpm5_0());
        Pmm(())
    }
}

pub trait GpioExt {
    type Gpio;

    fn constrain(self) -> Self::Gpio;
}

pub struct Output<TOK>(PhantomData<TOK>);
pub struct Input<PULL, INTR>(PhantomData<PULL>, PhantomData<INTR>);
pub struct Alternate1;
pub struct Alternate2;
pub struct Alternate3;

pub struct Pullup;
pub struct Pulldown;
pub struct Floating;
pub struct Unknown;
pub struct Enabled;
pub struct Disabled;

pub struct Locked;
pub struct Unlocked;

pub struct NoOutToken;
pub struct OutToken<'out>(PhantomData<&'out POUT>);

pub trait ConvertToInput {}
impl<TOK> ConvertToInput for Output<TOK> {}
impl ConvertToInput for Unknown {}

pub trait ConvertToOutput {}
impl<PULL> ConvertToOutput for Input<PULL, Disabled> {}
impl ConvertToOutput for Unknown {}

pub trait Known {}
impl Known for Floating {}
impl Known for Pulldown {}
impl Known for Pullup {}

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
        self.p1sel0.write(|w| unsafe { w.bits(0) });
        self.p1sel1.write(|w| unsafe { w.bits(0) });
        make_periph!(P1, self)
    }
}

impl<PULL, LOCK> P1<Input<PULL, Disabled>, LOCK> {
    pub fn pulldown(self) -> P1<Input<Pulldown, Disabled>, LOCK> {
        self.periph.p1out.write(|w| unsafe { w.bits(0x00) });
        self.periph.p1ren.write(|w| unsafe { w.bits(0xFF) });
        make_periph!(P1, self.periph)
    }

    pub fn pullup(self) -> P1<Input<Pullup, Disabled>, LOCK> {
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

// Don't need POUT token because we own the whole register already
impl P1<Output<NoOutToken>, Unlocked> {
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
    pub fn to_output(self) -> P1<Output<NoOutToken>, LOCK> {
        self.periph.p1dir.write(|w| unsafe { w.bits(0xFF) });
        make_periph!(P1, self.periph)
    }
}

impl<DIR> P1<DIR, Locked> {
    pub fn unlock(self, _lock: &Pmm) -> P1<DIR, Unlocked> {
        make_periph!(P1, self.periph)
    }
}

pub struct PSEL(());
pub struct POUT(());

impl<DIR, LOCK> P1<DIR, LOCK> {
    pub fn split(self) -> Parts<DIR, DIR, LOCK> {
        Parts {
            psel: PSEL(()),
            pout: POUT(()),
            p1_0: make_periph!(P1_0),
            p1_1: make_periph!(P1_1),
        }
    }
}

pub struct Parts<DIR0, DIR1, LOCK> {
    pub psel: PSEL,
    pub pout: POUT,
    pub p1_0: P1_0<DIR0, LOCK>,
    pub p1_1: P1_1<DIR1, LOCK>,
}

// P1.0
pub struct P1_0<DIR, LOCK> {
    _dir: PhantomData<DIR>,
    _lock: PhantomData<LOCK>,
}

impl<PULL: Known, INTR> P1_0<Input<PULL, INTR>, Unlocked> {
    pub fn read(&self) -> bool {
        unsafe { &*pac::P1::ptr() }.p1in.read().bits() & (1 << 0) != 0
    }
}

impl P1_0<Output<NoOutToken>, Unlocked> {
    pub fn enable<'out>(self, _pout: &'out POUT) -> P1_0<Output<OutToken<'out>>, Unlocked> {
        make_periph!(P1_0)
    }
}

impl<'out> P1_0<Output<OutToken<'out>>, Unlocked> {
    pub fn set_bit(&mut self) {
        unsafe { &*pac::P1::ptr() }
            .p1out
            .modify(|r, w| unsafe { w.bits(r.bits() | 1) });
    }

    pub fn clear_bit(&mut self) {
        unsafe { &*pac::P1::ptr() }
            .p1out
            .modify(|r, w| unsafe { w.bits(r.bits() & !1) });
    }
}

impl<DIR> P1_0<DIR, Locked> {
    pub fn unlock(self, _lock: &Pmm) -> P1_0<DIR, Unlocked> {
        make_periph!(P1_0)
    }
}

impl<DIR, LOCK> P1_0<DIR, LOCK> {
    pub fn alternate1(self, _psel: &PSEL) -> P1_0<Alternate1, LOCK> {
        let periph = unsafe { &*pac::P1::ptr() };
        periph.p1sel0.modify(|r, w| unsafe { w.bits(r.bits() | 1) });
        periph
            .p1sel1
            .modify(|r, w| unsafe { w.bits(r.bits() & !1) });
        make_periph!(P1_0)
    }

    pub fn alternate2(self, _psel: &PSEL) -> P1_0<Alternate2, LOCK> {
        let periph = unsafe { &*pac::P1::ptr() };
        periph
            .p1sel0
            .modify(|r, w| unsafe { w.bits(r.bits() & !1) });
        periph.p1sel1.modify(|r, w| unsafe { w.bits(r.bits() | 1) });
        make_periph!(P1_0)
    }

    pub fn alternate3(self, _psel: &PSEL) -> P1_0<Alternate3, LOCK> {
        let periph = unsafe { &*pac::P1::ptr() };
        periph.p1sel0.modify(|r, w| unsafe { w.bits(r.bits() | 1) });
        periph.p1sel1.modify(|r, w| unsafe { w.bits(r.bits() | 1) });
        make_periph!(P1_0)
    }
}

// P1.1
pub struct P1_1<DIR, LOCK> {
    _dir: PhantomData<DIR>,
    _lock: PhantomData<LOCK>,
}

impl<PULL: Known, INTR> P1_1<Input<PULL, INTR>, Unlocked> {
    pub fn read(&self) -> bool {
        unsafe { &*pac::P1::ptr() }.p1in.read().bits() & (1 << 0) != 0
    }
}

impl P1_1<Output<NoOutToken>, Unlocked> {
    pub fn enable<'out>(self, _pout: &'out POUT) -> P1_1<Output<OutToken<'out>>, Unlocked> {
        make_periph!(P1_1)
    }
}

impl<'out> P1_1<Output<OutToken<'out>>, Unlocked> {
    pub fn set_bit(&mut self) {
        unsafe { &*pac::P1::ptr() }
            .p1out
            .modify(|r, w| unsafe { w.bits(r.bits() | 1) });
    }

    pub fn clear_bit(&mut self) {
        unsafe { &*pac::P1::ptr() }
            .p1out
            .modify(|r, w| unsafe { w.bits(r.bits() & !1) });
    }
}

impl<DIR> P1_1<DIR, Locked> {
    pub fn unlock(self, _lock: &Pmm) -> P1_1<DIR, Unlocked> {
        make_periph!(P1_1)
    }
}

impl<DIR, LOCK> P1_1<DIR, LOCK> {
    pub fn alternate1(self, _psel: &PSEL) -> P1_1<Alternate1, LOCK> {
        let periph = unsafe { &*pac::P1::ptr() };
        periph.p1sel0.modify(|r, w| unsafe { w.bits(r.bits() | 1) });
        periph
            .p1sel1
            .modify(|r, w| unsafe { w.bits(r.bits() & !1) });
        make_periph!(P1_1)
    }

    pub fn alternate2(self, _psel: &PSEL) -> P1_1<Alternate2, LOCK> {
        let periph = unsafe { &*pac::P1::ptr() };
        periph
            .p1sel0
            .modify(|r, w| unsafe { w.bits(r.bits() & !1) });
        periph.p1sel1.modify(|r, w| unsafe { w.bits(r.bits() | 1) });
        make_periph!(P1_1)
    }

    pub fn alternate3(self, _psel: &PSEL) -> P1_1<Alternate3, LOCK> {
        let periph = unsafe { &*pac::P1::ptr() };
        periph.p1sel0.modify(|r, w| unsafe { w.bits(r.bits() | 1) });
        periph.p1sel1.modify(|r, w| unsafe { w.bits(r.bits() | 1) });
        make_periph!(P1_1)
    }
}

/**************************************************************************/
// Proxy stuff starts here

trait WritePdir {
    fn pdir_on(&self) -> bool;
}

trait WritePout {
    fn pout_on(&self) -> bool;
}

trait WritePren {
    fn pren_on(&self) -> bool;
}

macro_rules! make_proxy {
    ($Px:ident) => {
        $Px {
            _dir: PhantomData,
            _lock: PhantomData,
            out: false,
        }
    };

    ($Px:ident, $out:expr) => {
        $Px {
            _dir: PhantomData,
            _lock: PhantomData,
            out: $out,
        }
    };
}

pub struct P1_0Proxy<DIR, LOCK> {
    _dir: PhantomData<DIR>,
    _lock: PhantomData<LOCK>,
    out: bool,
}

impl<DIR, LOCK> WritePdir for P1_0Proxy<DIR, LOCK> {
    default fn pdir_on(&self) -> bool {
        false
    }
}
impl<DIR, LOCK> WritePout for P1_0Proxy<DIR, LOCK> {
    default fn pout_on(&self) -> bool {
        false
    }
}
impl<DIR, LOCK> WritePren for P1_0Proxy<DIR, LOCK> {
    default fn pren_on(&self) -> bool {
        false
    }
}

impl<LOCK> WritePout for P1_0Proxy<Output<NoOutToken>, LOCK> {
    fn pout_on(&self) -> bool {
        self.out
    }
}
impl<INTR, LOCK> WritePout for P1_0Proxy<Input<Pullup, INTR>, LOCK> {
    fn pout_on(&self) -> bool {
        true
    }
}

impl<LOCK> WritePdir for P1_0Proxy<Output<NoOutToken>, LOCK> {
    fn pdir_on(&self) -> bool {
        true
    }
}

impl<INTR, LOCK> WritePren for P1_0Proxy<Input<Pullup, INTR>, LOCK> {
    fn pren_on(&self) -> bool {
        true
    }
}
impl<INTR, LOCK> WritePren for P1_0Proxy<Input<Pulldown, INTR>, LOCK> {
    fn pren_on(&self) -> bool {
        true
    }
}

impl<PULL, LOCK> P1_0Proxy<Input<PULL, Disabled>, LOCK> {
    pub fn pulldown(self) -> P1_0Proxy<Input<Pulldown, Disabled>, LOCK> {
        make_proxy!(P1_0Proxy)
    }

    pub fn pullup(self) -> P1_0Proxy<Input<Pullup, Disabled>, LOCK> {
        make_proxy!(P1_0Proxy)
    }

    pub fn float(self) -> P1_0Proxy<Input<Floating, Disabled>, LOCK> {
        make_proxy!(P1_0Proxy)
    }
}

impl<DIR: ConvertToInput, LOCK> P1_0Proxy<DIR, LOCK> {
    pub fn to_input(self) -> P1_0Proxy<Input<Unknown, Disabled>, LOCK> {
        make_proxy!(P1_0Proxy)
    }
}

impl<DIR: ConvertToOutput, LOCK> P1_0Proxy<DIR, LOCK> {
    pub fn to_output(self) -> P1_0Proxy<Output<NoOutToken>, LOCK> {
        make_proxy!(P1_0Proxy, false)
    }
}

impl P1_0Proxy<Output<NoOutToken>, Unlocked> {
    pub fn on(self) -> P1_0Proxy<Output<NoOutToken>, Unlocked> {
        make_proxy!(P1_0Proxy, true)
    }

    pub fn off(self) -> P1_0Proxy<Output<NoOutToken>, Unlocked> {
        make_proxy!(P1_0Proxy, false)
    }
}

pub struct P1_1Proxy<DIR, LOCK> {
    _dir: PhantomData<DIR>,
    _lock: PhantomData<LOCK>,
    out: bool,
}

impl<DIR, LOCK> WritePdir for P1_1Proxy<DIR, LOCK> {
    default fn pdir_on(&self) -> bool {
        false
    }
}
impl<DIR, LOCK> WritePout for P1_1Proxy<DIR, LOCK> {
    default fn pout_on(&self) -> bool {
        false
    }
}
impl<DIR, LOCK> WritePren for P1_1Proxy<DIR, LOCK> {
    default fn pren_on(&self) -> bool {
        false
    }
}

impl<LOCK> WritePout for P1_1Proxy<Output<NoOutToken>, LOCK> {
    fn pout_on(&self) -> bool {
        self.out
    }
}
impl<INTR, LOCK> WritePout for P1_1Proxy<Input<Pullup, INTR>, LOCK> {
    fn pout_on(&self) -> bool {
        true
    }
}

impl<LOCK> WritePdir for P1_1Proxy<Output<NoOutToken>, LOCK> {
    fn pdir_on(&self) -> bool {
        true
    }
}

impl<INTR, LOCK> WritePren for P1_1Proxy<Input<Pullup, INTR>, LOCK> {
    fn pren_on(&self) -> bool {
        true
    }
}
impl<INTR, LOCK> WritePren for P1_1Proxy<Input<Pulldown, INTR>, LOCK> {
    fn pren_on(&self) -> bool {
        true
    }
}

impl<PULL, LOCK> P1_1Proxy<Input<PULL, Disabled>, LOCK> {
    pub fn pulldown(self) -> P1_1Proxy<Input<Pulldown, Disabled>, LOCK> {
        make_proxy!(P1_1Proxy)
    }

    pub fn pullup(self) -> P1_1Proxy<Input<Pullup, Disabled>, LOCK> {
        make_proxy!(P1_1Proxy)
    }

    pub fn float(self) -> P1_1Proxy<Input<Floating, Disabled>, LOCK> {
        make_proxy!(P1_1Proxy)
    }
}

impl<DIR: ConvertToInput, LOCK> P1_1Proxy<DIR, LOCK> {
    pub fn to_input(self) -> P1_1Proxy<Input<Unknown, Disabled>, LOCK> {
        make_proxy!(P1_1Proxy)
    }
}

impl<DIR: ConvertToOutput, LOCK> P1_1Proxy<DIR, LOCK> {
    pub fn to_output(self) -> P1_1Proxy<Output<NoOutToken>, LOCK> {
        make_proxy!(P1_1Proxy, false)
    }
}

impl P1_1Proxy<Output<NoOutToken>, Unlocked> {
    pub fn on(self) -> P1_1Proxy<Output<NoOutToken>, Unlocked> {
        make_proxy!(P1_1Proxy, true)
    }

    pub fn off(self) -> P1_1Proxy<Output<NoOutToken>, Unlocked> {
        make_proxy!(P1_1Proxy, false)
    }
}

pub struct BatchParts<DIR0, DIR1, LOCK> {
    pub p1_0: P1_0Proxy<DIR0, LOCK>,
    pub p1_1: P1_1Proxy<DIR1, LOCK>,
}

impl<DIR0, DIR1, LOCK> Parts<DIR0, DIR1, LOCK> {
    pub fn batch(self) -> BatchParts<DIR0, DIR1, LOCK> {
        BatchParts {
            p1_0: make_proxy!(P1_0Proxy),
            p1_1: make_proxy!(P1_1Proxy),
        }
    }
}

impl<DIR0, DIR1, LOCK> BatchParts<DIR0, DIR1, LOCK> {
    pub fn write(self) -> Parts<DIR0, DIR1, LOCK> {
        let mut pdir: u8 = 0;
        let mut pout: u8 = 0;
        let mut pren: u8 = 0;

        pdir |= self.p1_0.pdir_on() as u8;
        pout |= self.p1_0.pout_on() as u8;
        pren |= self.p1_0.pren_on() as u8;

        pdir |= (self.p1_1.pdir_on() as u8) << 1;
        pout |= (self.p1_1.pout_on() as u8) << 1;
        pren |= (self.p1_1.pren_on() as u8) << 1;

        let p1 = unsafe { &*pac::P1::ptr() };
        p1.p1dir.write(|w| unsafe { w.bits(pdir) });
        p1.p1out.write(|w| unsafe { w.bits(pout) });
        p1.p1ren.write(|w| unsafe { w.bits(pren) });

        Parts {
            psel: PSEL(()),
            pout: POUT(()),
            p1_0: make_periph!(P1_0),
            p1_1: make_periph!(P1_1),
        }
    }
}
