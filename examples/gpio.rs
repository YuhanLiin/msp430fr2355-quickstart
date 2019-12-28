#![no_main]
#![no_std]

extern crate panic_msp430;

use msp430::asm;
use msp430_rt::entry;
use msp430fr2355_quickstart::gpio::*;

fn delay(n: u32) {
    let mut i = 0;
    loop {
        asm::nop();

        i += 1;

        if i == n {
            break;
        }
    }
}

#[entry]
fn main() -> ! {
    const TIME: u32 = 100000;

    let periph = msp430fr2355::Peripherals::take().unwrap();
    let wdt = periph.WDT_A;
    wdt.wdtctl
        .write(|w| unsafe { w.wdtpw().bits(0x5A) }.wdthold().hold());

    let pmm = periph.PMM.freeze();
    let mut p1 = periph
        .P1
        .constrain()
        .to_input()
        .pullup()
        .pulldown()
        .float()
        .to_output()
        .unlock(&pmm)
        .to_input()
        .pullup()
        .enable_intr_rising_edge()
        .disable_intr()
        .to_output();

    p1.write(0xFF);
    delay(TIME);
    p1.write(0x00);
    delay(TIME);

    let batch = p1.split().batch();
    let p1_0 = batch.p1_0.to_input().to_output().on();
    let p1_1 = batch.p1_1.to_input().to_output().on();
    let parts = BatchParts { p1_0, p1_1 }.write();
    //delay(TIME);
    //let mut p1_0 = parts.p1_0.enable(&parts.pout);
    //p1_0.clear_bit();
    //delay(TIME);
    //p1_0.set_bit();

    loop {}
}
