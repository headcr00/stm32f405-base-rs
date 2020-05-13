use cortex_m::interrupt::Mutex;
use crate::stm32l4x6::interrupt;
use crate::shared_cell::SharedCell;
use crate::MY_SHARED_PER;
//PD5 PD6
static MY_SHARED_UART : Mutex<SharedCell<UartDma>> = Mutex::new(SharedCell::uninit());
static mut SINGLE_BORROW : bool = false;
struct UartDma
{
    buf: [u8; 128],
}

pub struct UartDmaWrapper
{
}

impl UartDmaWrapper {
    pub fn init() -> Option<Self>
    {
        if unsafe{SINGLE_BORROW == false}
        {
            unsafe{SINGLE_BORROW = true};
            cortex_m::interrupt::free(|cs| MY_SHARED_UART.borrow(cs).initialize(
                UartDma{
                    buf: [0; 128]
                };
            ));
            if let Some(per) = MY_SHARED_PER.get_value()
            {

                per.DMA1.cpar1.write(|addr| unsafe {addr.bits(my.buf.as_ptr() as usize as u32)});
            }
            Some(my)
        }
        else {
            None
        }
    }

    fn init_usart2(uart : &crate::stm32l4x6::USART2)
    {
        uart.cr1.write(|d| d.);

    }

    pub fn do_something(&self)
    {

    }

}

#[interrupt]
fn DMA1_CH6()
{
    /* USART 2 Rx handler */
    if let Some(per) = MY_SHARED_PER.get_value()
    {
        if per.DMA1.isr.read().tcif6().bit_is_set()
        {
            per.DMA1.ifcr.write(|d| d.ctcif6().set_bit());
        }

    }
}