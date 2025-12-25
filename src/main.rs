#![no_std]
#![no_main]

pub mod arithmetic;
pub mod cpu;
pub mod machine;
pub mod memory;

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use linked_list_allocator::LockedHeap;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::rcc;
use stm32f1xx_hal::serial;

use crate::machine::Machine;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

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
    let mut _serial = dp.USART2.serial(
        (usart_tx, usart_rx),
        serial::Config::default().baudrate(115200.bps()),
        &mut rcc,
    );

    let mut machine = Machine::default();
    machine.run();
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
