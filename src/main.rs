#![no_std]
#![no_main]

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m::peripheral::{syst};
use stm32f4::{
    stm32f405
};

use stm32f405::interrupt;
use stm32f405::Peripherals;
use cortex_m_rt::exception;
use cmim::{
    Move,
    Context,
    Exception,
};
use cmim::Context::Interrupt;
use core::borrow::{BorrowMut, Borrow};
use core::ops::Deref;


struct SharedData {
    something: u8,
   // timer_data_stealed: stm32f405::Peripherals,
    timer_data_normal: &stm32f405::Peripherals
}
static SHARED_DATA: Move<SharedData, stm32f4::stm32f405::Interrupt> =
    Move::new_uninitialized(Interrupt(stm32f405::Interrupt::TIM2));

#[entry]
fn main() -> ! {
    let peripherals = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut systick = peripherals.SYST;

    systick.set_clock_source(syst::SystClkSource::Core);
    systick.set_reload(1_000);

    systick.clear_current();
    systick.enable_counter();
    systick.enable_interrupt();

    let mut stm_peripherals = stm32f405::Peripherals::take().unwrap();
    SHARED_DATA.try_move(
        SharedData{
            something : 0,
            //timer_data_stealed: unsafe{stm32f405::Peripherals::steal()},
            timer_data_normal: &stm_peripherals.borrow()
        }
    ).ok();
    tim2_setup(&mut stm_peripherals);
    while !systick.has_wrapped() {
    }
    stm_peripherals.RCC.ahb1enr.write(|w| w.gpioden().bit(true));
    stm_peripherals.GPIOD.moder.write(|w| w.moder12().bits(1));

    loop {
        // your code goes here
         if let Some(value) = SHARED_DATA.try_lock(|data| data.something).ok()
         {
             if value > 64
             {
                 stm_peripherals.GPIOD.odr.write(|w| w.odr12().bit(true));
             }
             else {
                 stm_peripherals.GPIOD.odr.write(|w| w.odr12().bit(false));
             }
         }
    }
}

fn tim2_setup(per : &mut Peripherals)
{
    per.RCC.apb1rstr.write(|d| d.tim2rst().set_bit());
    per.RCC.apb1rstr.write(|d| d.tim2rst().clear_bit());

    per.RCC.apb1enr.write(|d| d.tim2en().set_bit());

    per.TIM2.dier.write(|d| d.uie().set_bit());
    per.TIM2.arr.write(|d|d.arr().bits(8000));
    per.TIM2.psc.write(|d| d.psc().bits(10));

    per.TIM2.cr1.write(|d|d.cen().set_bit());
    unsafe{
        cortex_m::peripheral::NVIC::unmask(stm32f405::interrupt::TIM2);
    }

}
#[interrupt]
fn TIM2()
{
    SHARED_DATA.try_lock(|mut data| {
        //data.timer_data.sr.write(|d| d.uif().clear_bit());
        if data.something < 128
        {
            data.something = data.something + 1;
        }
        else {
            data.something = 0;
        }
    }).ok();
}
#[exception]
fn SysTick()
{
    static mut COUNT: u32 = 0;
    *COUNT += 1;
}

