#![no_std]
use msp430fr2355::Peripherals;
use msp430fr2355_quickstart::{clocks::*, gpio::*, timer::*, watchdog::*};
use panic_msp430 as _;

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let wdt = periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();
    let mut p1 = periph.P1.constrain().to_output().unlock(&pmm);
    p1.write(0x00);

    let (_mclk, _smclk, aclk) = periph
        .CS
        .constrain()
        // Anything above this and flashing becomes a problem
        .mclk_dcoclk(1_000_000)
        .unwrap()
        .smclk_divide_1()
        .aclk_vloclk()
        .freeze();

    let mut timers = periph
        .TB0
        .constrain()
        .use_aclk(&aclk)
        .set_div(TimerDiv::_2)
        .set_div_ex(TimerDivEx::_3)
        .to_periodic();
    timers.sub_timer1.set_count(10000 / 6);
    timers.timer.start(5000);

    loop {
        while timers.sub_timer1.wait().is_none() {}
        p1.write(0x1);
        while timers.timer.wait().is_none() {}
        p1.write(0x0);
    }
}
