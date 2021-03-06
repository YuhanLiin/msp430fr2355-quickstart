#![no_main]
#![no_std]

extern crate msp430fr2355;
extern crate panic_msp430;

use core::cell::Cell;
use core::cell::RefCell;
use core::cell::UnsafeCell;
use msp430_rt::entry;

#[entry]
fn main() -> ! {
    let n = UnsafeCell::new(5);
    unsafe {
        *n.get() = 5;
    }

    let c = Cell::new(3);
    c.set(4);

    let r = RefCell::new(None);
    *r.borrow_mut() = Some(2);

    loop {}
}
