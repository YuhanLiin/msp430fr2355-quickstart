#![no_std]

extern crate panic_msp430;

use msp430fr2355_quickstart::{clocks::*, gpio::*, serial::*, watchdog::*};

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let wdt = periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();
    // Should be part of HAL API
    periph
        .P4
        .p4sel0
        .write(|w| unsafe { w.bits((1 << 3) | (1 << 2)) });

    let mut p1 = periph.P1.constrain().to_output().unlock(&pmm);
    p1.write(0x00);

    let (_mclk, smclk, aclk) = periph
        .CS
        .constrain()
        .mclk_dcoclk(2_000_000)
        .unwrap()
        .smclk_divide_1()
        .aclk_refoclk()
        .freeze();

    let (mut tx, rx) = periph
        .E_USCI_A1
        .constrain()
        .char_8bits()
        .lsb_first()
        .stopbits_1()
        .parity_none()
        .baudrate_smclk(9600, &smclk)
        .unwrap()
        .freeze();

    p1.write(0x1);

    // Echo loop
    loop {
        let ch = loop {
            if let Ok(ch) = rx.read() {
                break ch;
            }
        };
        while let Err(_) = tx.write(ch) {}
    }
}
