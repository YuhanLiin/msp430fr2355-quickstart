#![no_std]

extern crate panic_msp430;

use msp430fr2355_quickstart::{clocks::*, gpio::*, watchdog::*};

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let pmm = periph.PMM.freeze();
    let mut p1 = periph.P1.constrain().to_output().unlock(&pmm);
    p1.write(0x00);

    let (_mclk, smclk, _aclk) = periph
        .CS
        .constrain()
        .mclk_dcoclk(24000000)
        .unwrap()
        .smclk_divide_2()
        .aclk_vloclk()
        .freeze();

    let mut wdt = periph.WDT_A.constrain().set_smclk(&smclk).to_interval();
    wdt.start(WdtClkPeriods::_512K);

    while !wdt.wait_done() {}
    p1.write(0xFF);

    let mut wdt = wdt.to_watchdog();
    wdt.start(WdtClkPeriods::_512K);
}
