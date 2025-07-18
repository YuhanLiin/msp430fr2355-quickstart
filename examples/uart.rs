#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

use core::cell::RefCell;
use msp430::interrupt::enable as enable_interrupts;
use msp430_rt::entry;
use msp430fr2355::{interrupt, Peripherals, E_USCI_A1};
use critical_section::Mutex;

use panic_msp430 as _;

static PERIPHERALS: Mutex<RefCell<Option<Peripherals>>> =
    Mutex::new(RefCell::new(None));

// Print ASCII character synchronously, not meant to be called directly
fn transmit_byte(uart: &E_USCI_A1, ch: u8) {
    while uart.uca1ifg().read().uctxifg().is_uctxifg_0() {}
    uart.uca1txbuf().write(|w| unsafe { w.uctxbuf().bits(ch) });
}

fn transmit_char(uart: &E_USCI_A1, ch: u8) {
    match ch {
        b'\n' | b'\r' => {
            transmit_byte(uart, b'\r');
            transmit_byte(uart, b'\n');
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
    critical_section::with(|cs| {
        PERIPHERALS.borrow_ref_mut(cs).replace(peripherals);
    });
    unsafe { enable_interrupts() };
    loop {}
}

#[interrupt]
fn EUSCI_A1() {
    critical_section::with(|cs| {
        let Some(ref mut peripherals) = *PERIPHERALS.borrow_ref_mut(cs) else {return};
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
