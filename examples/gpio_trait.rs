#![no_std]

extern crate panic_msp430;

use msp430::asm;
use msp430fr2355::Peripherals;
use msp430fr2355_quickstart::gpio_trait::*;

fn main() {
    let peripherals = Peripherals::take().unwrap();

    // Disable watchdog
    let wdt = peripherals.WDT_A;
    wdt.wdtctl
        .write(|w| unsafe { w.wdtpw().bits(0x5A) }.wdthold().hold());

    peripherals.PMM.pm5ctl0.write(|w| w.locklpm5().locklpm5_0());

    let p1 = peripherals.P1;
    p1.pxdir().write(|w| unsafe { w.bits(1) });
    p1.pxout().write(|w| unsafe { w.bits(1) });

    loop {
        delay(10_000);

        // toggle outputs
        p1.pxselc().write(|w| unsafe { w.bits(1) });
    }
}

fn delay(n: u16) {
    let mut i = 0;
    loop {
        asm::nop();

        i += 1;

        if i == n {
            break;
        }
    }
}
