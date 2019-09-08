#![no_std]
#![feature(abi_msp430_interrupt)]

#[macro_use(interrupt)]
extern crate msp430fr2355;
extern crate panic_msp430;

use core::cell::RefCell;
use msp430::interrupt;
use msp430fr2355::Peripherals;

static PERIPHERALS: interrupt::Mutex<RefCell<Option<Peripherals>>> =
    interrupt::Mutex::new(RefCell::new(None));

fn main() {
    let peripherals = Peripherals::take().unwrap();
    let uart = &peripherals.E_USCI_A1;
    let p1 = &peripherals.P1;
    let p6 = &peripherals.P6;
    let p4 = &peripherals.P4;

    peripherals
        .WDT_A
        .wdtctl
        .write(|w| unsafe { w.wdtpw().bits(0x5A) }.wdthold().hold());
    peripherals.PMM.pm5ctl0.write(|w| w.locklpm5().locklpm5_0());

    // Set ACLK to REFO (32.768 kHz)
    peripherals.CS.csctl4.write(|w| w.sela().refoclk());

    p1.p1dir.write(|w| unsafe { w.bits(0xFF) });
    p6.p6dir.write(|w| unsafe { w.bits(0xFF) });
    p1.p1out.write(|w| unsafe { w.bits(0x00) });
    p6.p6out.write(|w| unsafe { w.bits(0x00) });

    uart.uca1ctlw0().write(|w| w.ucswrst().enable());
    {
        // Sets parity to even, keep everything else to default (8-bit data, LSB 1st, 1 stop bit)
        // Use ACLK (32.768 kHz) for clock source
        uart.uca1ctlw0()
            .write(|w| w.ucpen().set_bit().ucpar().even().ucssel().aclk());
        // Should setup a baud rate of 9600
        uart.uca1mctlw
            .write(|w| unsafe { w.ucos16().clear_bit().ucbrs().bits(0x92) });
        uart.uca1brw().write(|w| unsafe { w.bits(3) });
    }
    uart.uca1ctlw0().write(|w| w.ucswrst().disable());
    p4.p4sel0.write(|w| unsafe { w.bits((1 << 2) | (1 << 3)) });

    uart.uca1ie()
        .write(|w| w.uctxie().set_bit().ucrxie().set_bit());

    interrupt::free(|cs| {
        *PERIPHERALS.borrow(cs).borrow_mut() = Some(peripherals);
    });

    unsafe { interrupt::enable() };
    loop {}

    //for i in 0..100000 {
    //if i > 0 {
    //p6.p6out.modify(|r, w| unsafe { w.bits(!r.bits()) });
    //}
    //while uart.uca1ifg().read().uctxifg().is_uctxifg_0() {}
    //p1.p1out.modify(|r, w| unsafe { w.bits(!r.bits()) });
    //uart.uca1txbuf()
    //.write(|w| unsafe { w.uctxbuf().bits('H' as u8) });

    //while uart.uca1ifg().read().ucrxifg().is_ucrxifg_0() {}
    //p1.p1out.modify(|r, w| unsafe { w.bits(!r.bits()) });
    //p6.p6out.modify(|r, w| unsafe { w.bits(!r.bits()) });
    //uart.uca1rxbuf().read().ucrxbuf();
    //}
}

interrupt!(EUSCI_A1, uart_handler);
fn uart_handler() {
    interrupt::free(|cs| {
        let peripherals_ref = &*PERIPHERALS.borrow(cs).borrow();
        let peripherals = peripherals_ref.as_ref().unwrap();

        let p1 = &peripherals.P1;
        let p6 = &peripherals.P6;
        let uart = &peripherals.E_USCI_A1;

        let iv = uart.uca1iv().read().uciv();
        if iv.is_ucrxifg() {
            p6.p6out.modify(|r, w| unsafe { w.bits(!r.bits()) });
            uart.uca1rxbuf().read().ucrxbuf();
        } else if iv.is_uctxifg() {
            p1.p1out.modify(|r, w| unsafe { w.bits(!r.bits()) });
            uart.uca1txbuf()
                .write(|w| unsafe { w.uctxbuf().bits('H' as u8) });
        } else {
            p6.p6out.modify(|r, w| unsafe { w.bits(!r.bits()) });
            p1.p1out.modify(|r, w| unsafe { w.bits(!r.bits()) });
            p6.p6out.modify(|r, w| unsafe { w.bits(!r.bits()) });
            p1.p1out.modify(|r, w| unsafe { w.bits(!r.bits()) });
        }
    });
}
