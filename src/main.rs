#![no_std]
#![no_main]

pub mod arithmetic;
pub mod cpu;
pub mod executor;
pub mod machine;
pub mod memory;
pub mod protocol;
pub mod serial;
pub mod vmerror;

use core::cell::RefCell;
use core::panic::PanicInfo;
use cortex_m::interrupt;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use linked_list_allocator::LockedHeap;
use stm32h7::stm32h723::*;

use crate::machine::Machine;
use crate::serial::ProtocolSerial;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
pub static SERIAL: Mutex<RefCell<Option<ProtocolSerial>>> = Mutex::new(RefCell::new(None));

const USART_BAUD: u32 = 115200;
const USART_FREQ: u32 = 68_750_000;

#[entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 1024 * 10;
    let mut heap: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        ALLOCATOR.lock().init(heap.as_mut_ptr(), HEAP_SIZE);
    }

    let p = Peripherals::take().unwrap();

    let flash = p.FLASH;
    flash.acr().modify(|_, w| unsafe { w.latency().bits(2) });

    let rcc = p.RCC;

    //启用HSE
    rcc.cr().modify(|_, w| w.hseon().on());
    while rcc.cr().read().hserdy().is_not_ready() {}

    //配置 PLL1
    // DIVM1
    rcc.pllckselr()
        .modify(|_, w| unsafe { w.divm1().bits(2).pllsrc().hse() });

    rcc.pllcfgr().modify(|_, w| {
        w.divp1en()
            .enabled() // Enable PLL1_P
            .divq1en()
            .disabled() // Disable Q
            .divr1en()
            .disabled() // Disable R
            .pll1vcosel()
            .wide_vco() // Wide VCO range
            .pll1rge()
            .range8()
    });

    // DIVMN=44 DIVMP=2
    rcc.pll1divr()
        .modify(|_, w| unsafe { w.divn1().bits(44 - 1).divp1().bits(2 - 1) });

    // 启用 PLL1 并等待锁定
    rcc.cr().modify(|_, w| w.pll1on().set_bit());
    while rcc.cr().read().pll1rdy().is_not_ready() {}

    //配置总线分频（AHB, APB）
    rcc.d1cfgr().modify(|_, w| {
        w.d1cpre()
            .div1() // CPU SYSCLK
            .hpre()
            .div2() // AXI HCLK3
            .d1ppre()
            .div2() //APB3
    });

    rcc.d2cfgr().modify(|_, w| {
        w.d2ppre1()
            .div2() // APB1
            .d2ppre2()
            .div2() // APB2 
    });

    rcc.d3cfgr().modify(|_, w| {
        w.d3ppre().div2() // APB4
    });

    // 配置 USART2 的时钟源
    rcc.d2ccip2r().modify(|_, w| w.usart234578sel().rcc_pclk1());

    // 切换系统时钟源到 PLL1_P
    rcc.cfgr().modify(|_, w| w.sw().pll1());
    while !rcc.cfgr().read().sws().is_pll1() {} // Wait until switched

    //启用GPIOA和USART2
    rcc.ahb4enr()
        .modify(|_, w| w.gpioaen().enabled().gpiogen().enabled());
    rcc.apb1lenr().modify(|_, w| w.usart2en().enabled());

    let gpiog = p.GPIOG;
    gpiog.moder().modify(|_, w| w.moder7().output());
    gpiog.otyper().modify(|_, w| w.ot7().push_pull());
    gpiog.ospeedr().modify(|_, w| w.ospeedr7().low_speed());
    gpiog.pupdr().modify(|_, w| w.pupdr7().floating());
    gpiog.odr().write(|w| w.odr7().set_bit());

    let gpioa = p.GPIOA;

    // PA2: USART2_TX → Alternate Function 7
    // PA3: USART2_RX → Alternate Function 7
    gpioa
        .moder()
        .modify(|_, w| w.moder2().alternate().moder3().alternate());
    gpioa
        .otyper()
        .modify(|_, w| w.ot2().push_pull().ot3().push_pull()); // Push-pull
    gpioa
        .ospeedr()
        .modify(|_, w| w.ospeedr2().low_speed().ospeedr3().low_speed());
    gpioa
        .pupdr()
        .modify(|_, w| w.pupdr2().floating().pupdr3().floating()); // No pull
    gpioa.afrl().modify(|_, w| w.afr2().af7().afr3().af7()); // AF7 for USART2

    let usart = p.USART2;
    // 计算 BRR
    let brr = ((USART_FREQ as u64) << 4) / (16 * USART_BAUD as u64);
    // 设置字长 8-bit, 无奇偶校验
    usart.cr1().modify(|_, w| w.m0().bit8().pce().disabled());
    // 设置 1 停止位
    usart.cr2().modify(|_, w| w.stop().stop1());
    // 设置波特率
    usart.brr().write(|w| unsafe { w.brr().bits(brr as u16) });
    // 使能发送器、接收器、USART
    usart.cr1().modify(|_, w| {
        w.te()
            .enabled() // Transmitter enable
            .re()
            .enabled() // Receiver enable
            .ue()
            .enabled() // USART enable
    });

    interrupt::free(|cs| *SERIAL.borrow(cs).borrow_mut() = Some(ProtocolSerial {}));

    let mut machine = Machine::default();
    #[cfg(feature = "test")]
    {
        let test_code = include_bytes!("../tests/test.bin");
        for i in 0..test_code.len() {
            machine.write_memory(i as u32, test_code[i]).unwrap();
        }
    }
    machine.run();
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let gpiog = unsafe { GPIOG::ptr().as_ref() }.unwrap();
    gpiog.odr().write(|w| w.odr7().clear_bit());
    loop {}
}
