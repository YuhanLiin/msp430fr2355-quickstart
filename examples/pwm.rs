#![no_std]
use msp430fr2355::Peripherals;
use msp430fr2355_quickstart::{clocks::*, gpio::*, timer::*, watchdog::*};
use panic_msp430 as _;

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let wdt = periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();

    let p1 = periph.P1;
    p1.p1dir.write(|w| unsafe { w.bits(0xFF) });
    //P1.6 to TB0.1 and P1.7 to TB0.2
    p1.p1sel1.write(|w| unsafe { w.bits(1 << 6 | 1 << 7) });
    p1.p1out.write(|w| unsafe { w.bits(0x0) });

    let (_mclk, smclk, _aclk) = periph
        .CS
        .constrain()
        .mclk_dcoclk(1_000_000)
        .unwrap()
        .smclk_divide_1()
        .aclk_vloclk()
        .freeze();

    let mut pwms = periph.TB0.constrain().use_smclk(&smclk).to_pwm();

    pwms.set_period(1000);
    pwms.pwm1.set_duty(100);
    pwms.pwm2.set_duty(795);
    pwms.enable();

    loop {}
}
