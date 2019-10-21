use core::marker::PhantomData;
use msp430fr2355 as pac;
use pac::generic::{Readable, Reg, Writable};

type Reg8<T> = Reg<u8, T>;
type Reg16<T> = Reg<u16, T>;

pub trait GpioPeriph {
    type Pxin;
    type Pxout;
    type Pxdir;
    type Pxren;
    type Pxselc;
    type Pxsel0;
    type Pxsel1;

    fn pxin(&self) -> &Reg8<Self::Pxin>
    where
        Reg8<Self::Pxin>: Readable;

    fn pxout(&self) -> &Reg8<Self::Pxout>
    where
        Reg8<Self::Pxout>: Readable + Writable;

    fn pxdir(&self) -> &Reg8<Self::Pxdir>
    where
        Reg8<Self::Pxout>: Readable + Writable;

    fn pxren(&self) -> &Reg8<Self::Pxren>
    where
        Reg8<Self::Pxout>: Readable + Writable;

    fn pxselc(&self) -> &Reg8<Self::Pxselc>
    where
        Reg8<Self::Pxout>: Readable + Writable;

    fn pxsel0(&self) -> &Reg8<Self::Pxsel0>
    where
        Reg8<Self::Pxout>: Readable + Writable;

    fn pxsel1(&self) -> &Reg8<Self::Pxsel1>
    where
        Reg8<Self::Pxout>: Readable + Writable;
}

trait IntrPeriph: GpioPeriph {
    type Pxies;
    type Pxie;
    type Pxifg;
    type Pxiv;

    fn pxies(&self) -> &Reg8<Self::Pxies>
    where
        Reg8<Self::Pxout>: Readable + Writable;

    fn pxie(&self) -> &Reg8<Self::Pxie>
    where
        Reg8<Self::Pxout>: Readable + Writable;

    fn pxifg(&self) -> &Reg8<Self::Pxifg>
    where
        Reg8<Self::Pxout>: Readable + Writable;

    fn pxiv(&self) -> &Reg16<Self::Pxiv>
    where
        Reg16<Self::Pxout>: Readable + Writable;
}

pub trait GpioPin {
    type Periph: GpioPeriph;

    fn pin() -> u8;
}

trait Number {
    fn num() -> u8;
}
trait UnderSeven: Number {}
trait UnderFive: Number {}

struct Pin0;
impl Number for Pin0 {
    fn num() -> u8 {
        0
    }
}
impl UnderSeven for Pin0 {}
impl UnderFive for Pin0 {}

struct Pin1;
impl Number for Pin1 {
    fn num() -> u8 {
        1
    }
}
impl UnderSeven for Pin1 {}
impl UnderFive for Pin1 {}

struct Pin2;
impl Number for Pin2 {
    fn num() -> u8 {
        2
    }
}
impl UnderSeven for Pin2 {}
impl UnderFive for Pin2 {}

struct Pin3;
impl Number for Pin3 {
    fn num() -> u8 {
        3
    }
}
impl UnderSeven for Pin3 {}
impl UnderFive for Pin3 {}

struct Pin4;
impl Number for Pin4 {
    fn num() -> u8 {
        4
    }
}
impl UnderSeven for Pin4 {}
impl UnderFive for Pin4 {}

struct Pin5;
impl Number for Pin5 {
    fn num() -> u8 {
        5
    }
}
impl UnderSeven for Pin5 {}

struct Pin6;
impl Number for Pin6 {
    fn num() -> u8 {
        6
    }
}
impl UnderSeven for Pin6 {}

struct Pin7;
impl Number for Pin7 {
    fn num() -> u8 {
        7
    }
}

struct Port1<P>(PhantomData<P>);
impl<P: Number> GpioPin for Port1<P> {
    type Periph = pac::P1;

    fn pin() -> u8 {
        P::num()
    }
}

struct Port2<P>(PhantomData<P>);
impl<P: Number> GpioPin for Port2<P> {
    type Periph = pac::P2;

    fn pin() -> u8 {
        P::num()
    }
}

struct Port3<P>(PhantomData<P>);
impl<P: Number> GpioPin for Port3<P> {
    type Periph = pac::P3;

    fn pin() -> u8 {
        P::num()
    }
}

struct Port4<P>(PhantomData<P>);
impl<P: Number> GpioPin for Port4<P> {
    type Periph = pac::P4;

    fn pin() -> u8 {
        P::num()
    }
}

struct Port5<P>(PhantomData<P>);
impl<P: UnderFive> GpioPin for Port5<P> {
    type Periph = pac::P5;

    fn pin() -> u8 {
        P::num()
    }
}

struct Port6<P>(PhantomData<P>);
impl<P: UnderSeven> GpioPin for Port6<P> {
    type Periph = pac::P6;

    fn pin() -> u8 {
        P::num()
    }
}

macro_rules! gpio_impl {
    ($px:ident: $Px:ident =>
     $pxin:ident: $PxIN:ident, $pxout:ident: $PxOUT:ident, $pxdir:ident: $PxDIR:ident,
     $pxren:ident: $PxREN:ident, $pxselc:ident: $PxSELC:ident, $pxsel0:ident: $PxSEL0:ident, $pxsel1:ident: $PxSEL1:ident
     $(, [$pxies:ident: $PxIES:ident, $pxie:ident: $PxIE:ident, $pxifg:ident: $PxIFG:ident, $pxiv:ident: $PxIV:ident])?
    ) => {
        mod $px {
            use super::*;
            use pac::$px::{
                $PxDIR,  $PxOUT, $PxREN, $PxSEL0, $PxSEL1, $PxSELC, $PxIN,
                $($PxIE, $PxIES, $PxIFG, $PxIV)?
            };

            impl GpioPeriph for pac::$Px {
                type Pxdir = $PxDIR;
                type Pxin = $PxIN;
                type Pxout = $PxOUT;
                type Pxren = $PxREN;
                type Pxsel0 = $PxSEL0;
                type Pxsel1 = $PxSEL1;
                type Pxselc = $PxSELC;

                fn pxdir(&self) -> &Reg8<$PxDIR>
                where
                    Reg8<$PxDIR>: Readable + Writable,
                {
                    &self.$pxdir
                }

                fn pxin(&self) -> &Reg8<$PxIN>
                where
                    Reg8<$PxIN>: Readable,
                {
                    &self.$pxin
                }

                fn pxout(&self) -> &Reg8<$PxOUT>
                where
                    Reg8<$PxOUT>: Readable + Writable,
                {
                    &self.$pxout
                }

                fn pxren(&self) -> &Reg8<$PxREN>
                where
                    Reg8<$PxREN>: Readable + Writable,
                {
                    &self.$pxren
                }

                fn pxsel0(&self) -> &Reg8<$PxSEL0>
                where
                    Reg8<$PxSEL0>: Readable + Writable,
                {
                    &self.$pxsel0
                }

                fn pxsel1(&self) -> &Reg8<$PxSEL1>
                where
                    Reg8<$PxSEL1>: Readable + Writable,
                {
                    &self.$pxsel1
                }

                fn pxselc(&self) -> &Reg8<$PxSELC>
                where
                    Reg8<$PxSELC>: Readable + Writable,
                {
                    &self.$pxselc
                }
            }

            $(
                impl IntrPeriph for pac::$Px {
                    type Pxie = $PxIE;
                    type Pxies = $PxIES;
                    type Pxifg = $PxIFG;
                    type Pxiv = $PxIV;

                    fn pxie(&self) -> &Reg8<$PxIE>
                    where
                        Reg8<$PxIE>: Readable + Writable,
                    {
                        &self.$pxie
                    }

                    fn pxies(&self) -> &Reg8<$PxIES>
                    where
                        Reg8<$PxIES>: Readable + Writable,
                    {
                        &self.$pxies
                    }

                    fn pxifg(&self) -> &Reg8<$PxIFG>
                    where
                        Reg8<$PxIFG>: Readable + Writable,
                    {
                        &self.$pxifg
                    }

                    fn pxiv(&self) -> &Reg16<$PxIV>
                    where
                        Reg16<$PxIV>: Readable,
                    {
                        &self.$pxiv
                    }
                }
            )?
        }
    };
}

gpio_impl!(p1: P1 => p1in: _P1IN, p1out: _P1OUT, p1dir: _P1DIR,
     p1ren: _P1REN, p1selc: _P1SELC, 
     p1sel0: _P1SEL0, p1sel1: _P1SEL1, 
     [p1ies: _P1IES, p1ie: _P1IE, p1ifg: _P1IFG, p1iv: _P1IV]);

gpio_impl!(p2: P2 => p2in: _P2IN, p2out: _P2OUT, p2dir: _P2DIR,
     p2ren: _P2REN, p2selc: _P2SELC, 
     p2sel0: _P2SEL0, p2sel1: _P2SEL1, 
     [p2ies: _P2IES, p2ie: _P2IE, p2ifg: _P2IFG, p2iv: _P2IV]);

gpio_impl!(p3: P3 => p3in: _P3IN, p3out: _P3OUT, p3dir: _P3DIR,
     p3ren: _P3REN, p3selc: _P3SELC, 
     p3sel0: _P3SEL0, p3sel1: _P3SEL1, 
     [p3ies: _P3IES, p3ie: _P3IE, p3ifg: _P3IFG, p3iv: _P3IV]);

gpio_impl!(p4: P4 => p4in: _P4IN, p4out: _P4OUT, p4dir: _P4DIR,
     p4ren: _P4REN, p4selc: _P4SELC, 
     p4sel0: _P4SEL0, p4sel1: _P4SEL1, 
     [p4ies: _P4IES, p4ie: _P4IE, p4ifg: _P4IFG, p4iv: _P4IV]);

gpio_impl!(p5: P5 => p5in: _P5IN, p5out: _P5OUT, p5dir: _P5DIR,
     p5ren: _P5REN, p5selc: _P5SELC, 
     p5sel0: _P5SEL0, p5sel1: _P5SEL1);

gpio_impl!(p6: P6 => p6in: _P6IN, p6out: _P6OUT, p6dir: _P6DIR,
     p6ren: _P6REN, p6selc: _P6SELC, 
     p6sel0: _P6SEL0, p6sel1: _P6SEL1);
