#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

extern crate msp430fr2355;
extern crate panic_msp430;

use core::cell::RefCell;
use msp430::interrupt as mspint;
use msp430_rt::entry;
use msp430fr2355::{interrupt, Peripherals, E_USCI_A1};

static PERIPHERALS: mspint::Mutex<RefCell<Option<Peripherals>>> =
    mspint::Mutex::new(RefCell::new(None));

// Print ASCII character synchronously, not meant to be called directly
fn transmit_byte(uart: &E_USCI_A1, ch: u8) {
    while uart.uca1ifg().read().uctxifg().is_uctxifg_0() {}
    uart.uca1txbuf().write(|w| unsafe { w.uctxbuf().bits(ch) });
}

fn transmit_char(uart: &E_USCI_A1, ch: u8) {
    match ch as char {
        '\n' | '\r' => {
            transmit_byte(uart, '\r' as u8);
            transmit_byte(uart, '\n' as u8);
        }
        _ => transmit_byte(uart, ch),
    }
}

fn transmit_str(uart: &E_USCI_A1, s: &str) {
    for ch in s.bytes() {
        transmit_char(uart, ch);
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let uart = &peripherals.E_USCI_A1;
    let p4 = &peripherals.P4;

    // Turn off watchdog
    peripherals
        .WDT_A
        .wdtctl
        .write(|w| unsafe { w.wdtpw().bits(0x5A) }.wdthold().hold());
    peripherals.PMM.pm5ctl0.write(|w| w.locklpm5().locklpm5_0());

    // Set ACLK to REFO (32.768 kHz)
    peripherals.CS.csctl4.write(|w| w.sela().refoclk());

    p4.p4sel0.write(|w| unsafe { w.bits((1 << 3) | (1 << 2)) });
    {
        uart.uca1ctlw0().write(|w| w.ucswrst().enable());
        // Keep everything else to default (8-bit data, LSB 1st, 1 stop bit, no parity)
        // Use ACLK (32.768 kHz) for clock source to set baud rate of 9600
        uart.uca1mctlw
            .write(|w| unsafe { w.ucos16().clear_bit().ucbrs().bits(0x92) });
        uart.uca1brw().write(|w| unsafe { w.bits(3) });
        // Need to set CTLW0 here or else this write will wipe out previous CTLW0 settings, such as
        // the clock setting, causing everything to go to hell
        uart.uca1ctlw0()
            .write(|w| w.ucswrst().disable().ucssel().aclk());
    }

    transmit_str(uart, "hello world\n");

    uart.uca1ie().write(|w| w.ucrxie().set_bit());
    mspint::free(|cs| {
        *PERIPHERALS.borrow(cs).borrow_mut() = Some(peripherals);
    });
    unsafe { mspint::enable() };
    loop {}
}

#[interrupt]
fn EUSCI_A1() {
    mspint::free(|cs| {
        let peripherals_ref = &*PERIPHERALS.borrow(cs).borrow();
        let peripherals = peripherals_ref.as_ref().unwrap();
        let uart = &peripherals.E_USCI_A1;

        let iv = uart.uca1iv().read().uciv();
        if iv.is_ucrxifg() {
            // Echo the input back
            let c = uart.uca1rxbuf().read().ucrxbuf().bits();
            transmit_char(uart, c);
        } else {
            transmit_str(uart, "WTF wrong interrupt\n");
        }
    });
}
