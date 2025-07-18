#![no_main]
#![no_std]

// Always include the PAC so the linker can populate the interrupt vector table.
// The PAC offers a register-level interface for interacting with periperals
use msp430fr2355 as _;

// Include an infinitely-looping panic handler.
use panic_msp430 as _; 

use msp430_rt::entry;

#[entry]
fn main() -> ! {
    // Your initialisation code here
    loop {
        // Your code here
    }
}
