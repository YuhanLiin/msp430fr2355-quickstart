#![no_std]
use msp430fr2355_quickstart::{clocks::*, gpio::*, serial::*, timer::*, watchdog::*};
use panic_msp430 as _;

fn main() {
    let periph = msp430fr2355::Peripherals::take().unwrap();

    let wdt = periph.WDT_A.constrain();

    let pmm = periph.PMM.freeze();

    let p1 = periph.P1;
    //P1.6 to TB0.CC1A
    p1.p1sel1.write(|w| unsafe { w.bits(1 << 6) });
    p1.p1dir.write(|w| unsafe { w.bits(1) });
    p1.p1out.write(|w| unsafe { w.bits(0) });

    //For UART
    periph
        .P4
        .p4sel0
        .write(|w| unsafe { w.bits((1 << 3) | (1 << 2)) });

    let (_mclk, smclk, aclk) = periph
        .CS
        .constrain()
        .mclk_dcoclk(1_000_000)
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

    let mut captures = periph
        .TB0
        .constrain()
        .use_aclk(&aclk)
        .set_div(TimerDiv::_2)
        .set_div_ex(TimerDivEx::_3)
        .config_capture()
        .config_chan1(CaptureMode::Falling, CaptureSelect::CapInputA)
        .freeze();

    p1.p1out.write(|w| unsafe { w.bits(1) });
    write(&mut tx, 'x');
    write(&mut tx, 'x');
    write(&mut tx, 'x');
    write(&mut tx, 'x');

    let mut last_cap = 0;
    loop {
        match captures.capture1.capture() {
            Err(_) => {
                write(&mut tx, '!');
                write(&mut tx, '\n');
            }
            Ok(Some(cap)) => {
                let diff = cap.wrapping_sub(last_cap);
                last_cap = cap;
                print_num(&mut tx, diff);
            }
            Ok(None) => {}
        }
    }
}

fn print_num(tx: &mut Tx, num: u16) {
    write(tx, '0');
    write(tx, 'x');
    print_hex(tx, num >> 12);
    print_hex(tx, (num >> 8) & 0xF);
    print_hex(tx, (num >> 4) & 0xF);
    print_hex(tx, num & 0xF);
    write(tx, '\n');
}

fn print_hex(tx: &mut Tx, h: u16) {
    let c = match h {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        15 => 'f',
        _ => '?',
    };
    write(tx, c);
}

fn write(tx: &mut Tx, ch: char) {
    while let Err(_) = tx.write(ch as u8) {}
}
