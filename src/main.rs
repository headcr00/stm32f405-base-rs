#![no_std]
#![no_main]
use core::{
    cell::UnsafeCell,
    cmp::PartialEq,
    mem::MaybeUninit,
    result::Result,
    sync::atomic::{AtomicU8, Ordering},
};
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
mod shared_cell;
use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m::peripheral::{syst};
use stm32l4::{
    stm32l4x6
};

use stm32l4x6::interrupt;
use stm32l4x6::Peripherals;
use cortex_m_rt::exception;
use cmim::{
    Move,
    Context,
    Exception,
};
use cmim::Context::Interrupt;
use core::borrow::{BorrowMut, Borrow};
use core::ops::Deref;

static MY_SHARED_VAR: shared_cell::SharedCell<SharedData> = shared_cell::SharedCell::uninit();

struct SharedData {
    something: u16,
}
#[entry]
fn main() -> ! {
    let peripherals = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut systick = peripherals.SYST;
    MY_SHARED_VAR.initialize(SharedData{
        something : 0,
        // timer_data_stealed: unsafe{stm32l4x6::Peripherals::steal()},
        // timer_data_normal: &stm_peripherals.borrow()
    });
    systick.set_clock_source(syst::SystClkSource::Core);
    systick.set_reload(1_000);

    systick.clear_current();
    systick.enable_counter();
    systick.enable_interrupt();


    let mut stm_peripherals = stm32l4x6::Peripherals::take().unwrap();
    tim2_setup(&mut stm_peripherals);
    while !systick.has_wrapped() {
    }
    

    stm_peripherals.RCC.ahb2enr.write(|w| w.gpioben().set_bit());
    stm_peripherals.GPIOB.moder.write(|w| w.moder2().output());

    loop {
        // your code goes here

         if let Some(value) = MY_SHARED_VAR.get_value().as_ref(){
                 if value.something > 250
                 {
                     stm_peripherals.GPIOB.odr.write(|w| w.odr2().set_bit());
                 } else {
                     stm_peripherals.GPIOB.odr.write(|w| w.odr2().clear_bit());
                 }
         }
    }
}

fn tim2_setup(per : &mut Peripherals)
{

    per.RCC.apb1enr1.write(|d| d.tim2en().set_bit());
    per.TIM2.dier.write(|d| d.uie().set_bit());
    per.TIM2.arr.write(|d|d.arr().bits(8000));
    per.TIM2.psc.write(|d| d.psc().bits(10));

    per.TIM2.cr1.write(|d|d.cen().set_bit());
    // unsafe{
    //     cortex_m::peripheral::NVIC::unmask(stm32l4x6::interrupt::TIM2);
    // }

}
#[interrupt]
fn TIM2()
{
}
#[exception]
fn SysTick()
{
    MY_SHARED_VAR.modify(|d| {
        d.something = d.something + 1;
        if d.something == 750
        {
            d.something = 0;
        }
    });
}
/*
pub fn try_get(&self) -> Result<Option<T>, ()> {
    match self.context{
        Self::LOCKED => Ok(None),
        Self::INIT_AND_IDLE =>{
            self.state.store(Self::LOCKED, Ordering::SeqCst);
            let old = unsafe {
                // Get a pointer to the initialized data
                let mu_ptr = self.data.get();
            };
            self.state.store(Self::INIT_AND_IDLE, Ordering::SeqCst);
            Ok(old)
        }
        Self::UNINIT => Err(())
    }
}*/