#![no_std]
#![no_main]

pub mod cpu;
pub mod decode;
pub mod execute;
pub mod machine;
pub mod memory;
pub mod protocol;
pub mod register;
pub mod vm_exception;

use crate::machine::Machine;
use core::{cell::RefCell, panic::PanicInfo};
use cortex_m::interrupt::{self, Mutex};
use cortex_m_rt::{ExceptionFrame, entry, exception};
use linked_list_allocator::LockedHeap;
use object::{File, Object, ObjectSection};
use stm32h7xx_hal::{
    pac::{Peripherals, USART2},
    prelude::*,
    serial::Serial,
};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

type ProtocolSerial = Serial<USART2>;
pub static SERIAL: Mutex<RefCell<Option<ProtocolSerial>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 1024 * 100;
    let mut heap: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        ALLOCATOR.lock().init(heap.as_mut_ptr(), HEAP_SIZE);
    }

    let dp = Peripherals::take().unwrap();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(160.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);

    let tx = gpioa.pa2.into_alternate();
    let rx = gpioa.pa3.into_alternate();

    let serial = dp
        .USART2
        .serial((tx, rx), 115200.bps(), ccdr.peripheral.USART2, &ccdr.clocks)
        .unwrap();

    interrupt::free(|cs| *SERIAL.borrow(cs).borrow_mut() = Some(serial));

    let mut machine = Machine::default();
    if cfg!(feature = "test") {
        machine.cpu.xregs[2] = 1024; // sp
        load_elf(&mut machine, include_bytes!("../tests/test.elf"));
    }
    machine.run();
}

fn load_elf(machine: &mut Machine, file_data: &[u8]) {
    let file = File::parse(file_data).unwrap();
    for section in file.sections() {
        let address = section.address() as usize;
        for (i, v) in section.data().unwrap().iter().enumerate() {
            machine.memory.write(address + i, *v).unwrap();
        }
    }
    machine.cpu.pc = file.entry() as u32;
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[exception]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    loop {}
}
