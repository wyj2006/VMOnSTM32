#![no_std]
#![no_main]

pub mod arithmetic;
pub mod cpu;
pub mod executor;
pub mod machine;
pub mod memory;
pub mod protocol;
pub mod vmerror;

use core::cell::RefCell;
use core::panic::PanicInfo;
use cortex_m::interrupt;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use linked_list_allocator::LockedHeap;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::rcc;
use stm32f1xx_hal::serial;
use stm32f1xx_hal::serial::Serial;

use crate::machine::Machine;

pub type SerialType = Serial<pac::USART2>;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
pub static SERIAL: Mutex<RefCell<Option<SerialType>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 1024;
    let mut heap: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        ALLOCATOR.lock().init(heap.as_mut_ptr(), HEAP_SIZE);
    }

    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.freeze(rcc::Config::hse(8.MHz()), &mut flash.acr);

    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let usart_tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let usart_rx = gpioa.pa3;
    let serial = dp.USART2.serial(
        (usart_tx, usart_rx),
        serial::Config::default().baudrate(115200.bps()),
        &mut rcc,
    );
    interrupt::free(|cs| {
        *SERIAL.borrow(cs).borrow_mut() = Some(serial);
    });

    let mut machine = Machine::default();
    machine.run();
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
