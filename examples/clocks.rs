#![no_std]

extern crate panic_msp430;

use msp430fr2355_quickstart::{clocks::*, gpio::*, watchdog::*};

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let pmm = periph.PMM.freeze();
    let mut p1 = periph.P1.constrain().to_output().unlock(&pmm);
    p1.write(0x00);

    let clocks = periph
        .CS
        .constrain()
        .mclk_dcoclk(4000000)
        .unwrap()
        .smclk_on(2000000)
        .aclk_vloclk();
    let clocks = clocks.freeze();

    let mut wdt = periph
        .WDT_A
        .constrain()
        .set_smclk(&clocks)
        .unwrap()
        .to_interval();
    wdt.start(WdtClkPeriods::_512K);

    while !wdt.wait_done() {}
    p1.write(0xFF);

    let mut wdt = wdt.to_watchdog();
    wdt.start(WdtClkPeriods::_512K);
}
