#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use msp430_rt::entry;
use msp430fr2355::interrupt;
use msp430_atomic::AtomicU16;

use panic_msp430 as _;

#[entry]
fn main() -> ! {
    loop {}
}

static X: AtomicU16 = AtomicU16::new(0);

#[interrupt]
fn DefaultHandler() {
    X.store(X.load()+1)
}
