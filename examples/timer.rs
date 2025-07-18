#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use msp430::interrupt::enable as enable_interrupts;
use msp430_rt::entry;
use msp430fr2355::{interrupt, Peripherals};
use critical_section::Mutex;

use core::cell::RefCell;

use panic_msp430 as _;

static PERIPHERALS: Mutex<RefCell<Option<Peripherals>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    // Disable watchdog
    let wdt = &peripherals.WDT_A;
    wdt.wdtctl
        .write(|w| unsafe { w.wdtpw().bits(0x5A) }.wdthold().hold());

    // GPIOs don't work without this line
    peripherals.PMM.pm5ctl0.write(|w| w.locklpm5().locklpm5_0());

    let port1 = &peripherals.P1;
    let port6 = &peripherals.P6;

    // Enable LEDs as outputs
    port1.p1dir.write(|w| unsafe { w.bits(0xFF) });
    port6.p6dir.write(|w| unsafe { w.bits(0xFF) });
    // Turn on green LED, turn off red LED
    port1.p1out.write(|w| unsafe { w.bits(0x00) });
    port6.p6out.write(|w| unsafe { w.bits(0xFF) });

    let timer = &peripherals.TB0;
    // Set upmode interval to 1200 cycles
    timer.tb0ccr0.write(|w| unsafe { w.bits(10000) });
    timer.tb0ex0.write(|w| w.tbidex()._4());
    // Set timer0 to use ACLK, up mode, and enable TBIFG interrupts
    timer.tb0ctl
        .write(|w| w.tbssel().aclk().mc().up().tbclr().set_bit());
    timer.tb0cctl0.write(|w| w.ccie().set_bit());
    critical_section::with(|cs| {
        PERIPHERALS.borrow_ref_mut(cs).replace(peripherals);
    });

    unsafe { enable_interrupts() };

    loop {}
}

#[interrupt]
fn TIMER0_B0() {
    critical_section::with(|cs| {
        let Some(ref mut peripherals) = *PERIPHERALS.borrow_ref_mut(cs) else {return};

        // Clearing the IFG bit causes interrupt to stop working, so we don't do it
        //let timer = &peripherals.TB0;
        //timer.tb0cctl0.write(|w| w.ccifg().clear_bit());

        let port1 = &peripherals.P1;
        let port6 = &peripherals.P6;
        // Toggle LEDs
        port1.p1out.modify(|r, w| unsafe { w.bits(!r.bits()) });
        port6.p6out.modify(|r, w| unsafe { w.bits(!r.bits()) });
    });
}
