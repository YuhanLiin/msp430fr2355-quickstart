#![no_main]
#![no_std]

use msp430fr2355 as _;
use msp430_rt::entry;

use core::cell::Cell;
use core::cell::RefCell;
use core::cell::UnsafeCell;

use panic_msp430 as _;

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
