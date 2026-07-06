#![no_std]
#![no_main]

use cortex_m::peripheral;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_stm32::peripherals::{USART1, DMA1_CH4, DMA1_CH5};
use embassy_stm32::{Config, bind_interrupts};
use embassy_stm32::usart::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, BufferedUartTx, Config as UConfig};
use embassy_time::{Duration, Timer};

bind_interrupts!(struct Irqs
{
    USART1 => BufferedInterruptHandler<USART1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner)
{
    let p = embassy_stm32::init(Default::default());
    let mut tx_buf = [0u8; 92];
    let mut rx_buf = [0u8; 92];
    //burada static kullan
    let buart = BufferedUart::new(p.USART1, p.DMA1_CH5, p.DMA1_CH4, &mut tx_buf, &mut rx_buf, Irqs, UConfig::default());
    let (buart_tx, buart_rx) = buart.split();

}
