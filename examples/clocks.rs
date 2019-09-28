#![no_std]

extern crate panic_msp430;

use msp430fr2355_quickstart::{clocks::*, gpio::*, watchdog::*};

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let wdt = periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();
    let mut p1 = periph.P1.constrain().to_output().unlock(&pmm);
    p1.write(0x00);

    let (_mclk, smclk, _aclk) = periph
        .CS
        .constrain()
        // Anything above this and flashing becomes a problem
        .mclk_dcoclk(18000000)
        .unwrap()
        .smclk_divide_1()
        .aclk_vloclk()
        .freeze();

    let mut wdt = wdt.set_smclk(&smclk).to_interval();
    wdt.start(WdtClkPeriods::_8192K);

    while !wdt.wait_done() {}

    let mut wdt = wdt.to_watchdog();
    wdt.start(WdtClkPeriods::_8192K);
    p1.write(0xFF);

    loop {
        msp430::asm::nop();
    }
}
