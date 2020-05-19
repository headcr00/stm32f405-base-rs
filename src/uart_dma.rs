use crate::shared_cell::SharedCell;
use crate::stm32l4x6::interrupt;
use crate::MY_SHARED_PER;
use core::borrow::Borrow;
use cortex_m::interrupt::Mutex;
use stm32l4::stm32l4x6::usart1::RegisterBlock;

//PD5 PD6
static MY_SHARED_UART: Mutex<SharedCell<UartDma>> = Mutex::new(SharedCell::uninit());
static mut SINGLE_BORROW: bool = false;
struct UartDma {
    buf: [u8; 32],
    pos: usize,
}

pub struct UartDmaWrapper {}

impl UartDmaWrapper {
    pub fn init() -> Option<Self> {
        if unsafe { SINGLE_BORROW == false } {
            unsafe { SINGLE_BORROW = true };
            cortex_m::interrupt::free(|cs| {
                MY_SHARED_UART.borrow(cs).initialize(UartDma {
                    buf: [0; 32],
                    pos: 0,
                });
            });
            if let Some(p) = MY_SHARED_PER.get_value() {
                p.RCC.ahb2enr.modify(|_, w| w.gpioden().set_bit());
                p.RCC.apb1enr1.modify(|_, w| w.usart2en().set_bit());
                p.GPIOD.pupdr.modify(|_, w| {
                    w.pupdr5().pull_up();
                    w.pupdr6().pull_up()
                });

                p.GPIOD.moder.modify(|_, w| {
                    w.moder5().alternate();
                    w.moder6().alternate()
                });
                p.GPIOD.ospeedr.modify(|_, w| {
                    w.ospeedr5().very_high_speed();
                    w.ospeedr6().very_high_speed()
                });

                p.GPIOD.otyper.modify(|_, w| {
                    w.ot5().push_pull();
                    w.ot6().push_pull()
                });

                p.GPIOD.afrl.modify(|_, w| {
                    w.afrl5().af7();
                    w.afrl6().af7()
                });

                UartDmaWrapper::init_usart2(&p.USART2, 9600);
                UartDmaWrapper::init_usart_dma(&p);
            }
            Some(UartDmaWrapper {})
        } else {
            None
        }
    }

    fn init_usart2(uart: &crate::stm32l4x6::USART2, speed: u32) {
        let brr = 4_000_000 / (speed);
        uart.brr.write(|d| d.brr().bits(brr as u16));
        uart.cr3.modify(|_, w| w.dmar().enabled());
        uart.cr1.write(|d| {
            d.m0().bit8();
            d.re().enabled();
            d.te().enabled();
            d.ue().enabled()
        });
    }

    fn init_usart_dma(p: &crate::stm32l4x6::Peripherals) {
        p.RCC.ahb1enr.modify(|_, w| w.dma1en().set_bit());
        p.DMA1
            .cpar6
            .write(|w| w.pa().bits(&p.USART2.rdr as *const _ as u32));
        cortex_m::interrupt::free(|cs| {
            MY_SHARED_UART.borrow(cs).modify(|uart_s| {
                p.DMA1
                    .cmar6
                    .write(|w| w.ma().bits(&uart_s.buf as *const u8 as u32));
                p.DMA1
                    .cndtr6
                    .write(|w| w.ndt().bits(unsafe { uart_s.buf.len() as u16 }));
            });
        });
        p.DMA1.cselr.write(|w| w.c6s().map2());
        p.DMA1.ccr6.write(|w| {
            w.circ().disabled();
            w.dir().from_peripheral();
            w.minc().enabled();
            w.pinc().disabled();
            w.msize().bits8();
            w.tcie().enabled()
        });
        p.DMA1.ccr6.modify(|_, w| w.en().enabled());
        unsafe {
            cortex_m::peripheral::NVIC::unmask(crate::stm32l4x6::interrupt::DMA1_CH6);
        }
    }

    pub fn do_something(&self) -> bool {
        if let Some(p) = MY_SHARED_PER.get_value() {
            if p.USART2.isr.read().rxne().bit_is_set() == true {
                cortex_m::interrupt::free(|cs| {
                    MY_SHARED_UART
                        .borrow(cs)
                        .modify(|buf| {
                            let rchar = (p.USART2.rdr.read().bits() & 0xFF) as u8;
                            if buf.pos >= buf.buf.len() {
                                buf.pos = 0;
                            }
                            buf.buf[buf.pos] = rchar;
                            buf.pos = buf.pos + 1;
                        })
                        .ok()
                });
                return true;
            }
        }
        false
    }
}

#[interrupt]
fn DMA1_CH6() {
    /* USART 2 Rx handler */
    if let Some(per) = MY_SHARED_PER.get_value() {
        if per.DMA1.isr.read().tcif6().bit_is_set() {
            per.DMA1.ifcr.write(|d| d.ctcif6().set_bit());
            cortex_m::interrupt::free(|cs| {
                if let Some(data) = MY_SHARED_UART.borrow(cs).get_value() {
                    for i in data.buf.iter() {
                        while per.USART2.isr.read().txe().bit_is_clear() {}
                        per.USART2.tdr.write(|f| unsafe { f.tdr().bits(*i as u16) });
                    }
                }
            });
            per.GPIOB.odr.modify(|_, w| w.odr2().set_bit());
        }
    }
}
