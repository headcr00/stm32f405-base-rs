#![no_std]
#![no_main]
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
mod shared_cell;
use cortex_m_rt::entry;
use cortex_m::peripheral::{syst};
use stm32l4::{
    stm32l4x6
};
use shared_cell::SharedCell;

use stm32l4x6::interrupt;

use stm32l4x6::Peripherals;
use cortex_m_rt::exception;
use cortex_m::interrupt::Mutex;

static MY_SHARED_VAR: shared_cell::SharedCell<SharedData> = shared_cell::SharedCell::uninit();
static MY_SHARED_PER_MUTEXD: Mutex<SharedCell<stm32l4x6::Peripherals>> = Mutex::new(SharedCell::uninit());

struct SharedData {
    something: u16,
}
#[entry]
fn main() -> ! {
    let peripherals = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut systick = peripherals.SYST;
    let  stm_peripherals = stm32l4x6::Peripherals::take().unwrap();

    cortex_m::interrupt::free(|cs| MY_SHARED_PER_MUTEXD.borrow(cs).initialize(stm_peripherals));

    MY_SHARED_VAR.initialize(SharedData{
        something : 0,
    });
    systick.set_clock_source(syst::SystClkSource::Core);
    systick.set_reload(1_000);

    systick.clear_current();
    systick.enable_counter();
    systick.enable_interrupt();


    //tim2_setup(&mut stm_peripherals);
    while !systick.has_wrapped() {
    }
    cortex_m::interrupt::free(|cs|{
        if let Some(pers) = MY_SHARED_PER_MUTEXD.borrow(cs).get_value()
        {
            pers.RCC.ahb2enr.write(|w| w.gpioben().set_bit());
            pers.GPIOB.moder.write(|w| w.moder2().output());
            tim2_setup(&pers);
        }
    });




    loop {
        // your code goes here
         if let Some(value) = MY_SHARED_VAR.get_value().as_ref(){
            cortex_m::interrupt::free( |cs| {
                if let Some(pers) = MY_SHARED_PER_MUTEXD.borrow(cs).get_value()
                {
                    if value.something > 250
                    {
                    pers.GPIOB.odr.write(|w| w.odr2().set_bit());
                    } else {
                    pers.GPIOB.odr.write(|w| w.odr2().clear_bit());
                    }
                }
            });
        }
    }
}

fn tim2_setup(per : &Peripherals)
{

    per.RCC.apb1enr1.write(|d| d.tim2en().set_bit());
    per.TIM2.dier.write(|d| d.uie().set_bit());
    per.TIM2.arr.write(|d|d.arr().bits(100));
    per.TIM2.psc.write(|d| d.psc().bits(10));

    per.TIM2.cr1.write(|d|d.cen().set_bit());
    unsafe{
        cortex_m::peripheral::NVIC::unmask(stm32l4x6::interrupt::TIM2);
    }

}
#[interrupt]
fn TIM2()
{
    cortex_m::interrupt::free( |cs| {
        if let Some(p) = MY_SHARED_PER_MUTEXD.borrow(cs).get_value()
        {
            p.TIM2.sr.write(|d| d.uif().clear_bit());
            MY_SHARED_VAR.modify(|d| {
                d.something = d.something + 1;
                if d.something == 750
                {
                    d.something = 0;
                }
            }).ok();
        }
    });


}
#[exception]
fn SysTick()
{
    // MY_SHARED_VAR.modify(|d| {
    //     d.something = d.something + 1;
    //     if d.something == 750
    //     {
    //         d.something = 0;
    //     }
    // }).ok();
}
