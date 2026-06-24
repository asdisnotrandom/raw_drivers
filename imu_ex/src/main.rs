#![no_std]
#![no_main]

use embassy_rp::i2c::{Async, I2c};
use embassy_rp::{bind_interrupts};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    I2C0_IRQ => embassy_rp::i2c::InterruptHandler<embassy_rp::peripherals::I2C0>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>)
{
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
#[embassy_executor::task]
async fn mpu_task(mut i2c: I2c<'static, embassy_rp::peripherals::I2C0, Async>)
{
    let REG_ADDR: u8 = 0x68;
    const ACCEL_SCALE: f32 = 9.81 / 16384.0;

     match i2c.write_async(REG_ADDR, [0x6B, 0x00]).await {
        Ok(_) => {
            log::info!("Imu sensoru devrede");
        }
        Err(e) => {
            log::error!("Imu baslamadi: {:?}", e);
            return; // Hata varsa çık
        }
    }
    loop {
        let mut buff = [0u8; 14];
        match i2c.write_read_async(REG_ADDR, [0x3B], &mut buff).await {
            Ok(_) => {
                let ax = i16::from_be_bytes([buff[0], buff[1]]);
                let ay = i16::from_be_bytes([buff[2], buff[3]]);
                let az = i16::from_be_bytes([buff[4], buff[5]]);
                let gx = i16::from_be_bytes([buff[8], buff[9]]);
                let gy = i16::from_be_bytes([buff[10], buff[11]]);
                let gz = i16::from_be_bytes([buff[12], buff[13]]);
                log::info!("X: {:.2}g, Y: {:.2}g, Z: {:.2}g", ax as f32 * ACCEL_SCALE, ay as f32 * ACCEL_SCALE, az as f32 *ACCEL_SCALE);
                log::info!("GX: {}, GY: {}, GZ: {}", gx, gy, gz);
            }
            Err(e) => {
                log::error!("I2C okuma hatası: {:?}", e);
            }
        }
        Timer::after_millis(500).await;
    }
}
#[embassy_executor::main]
async fn main(spawner: Spawner)
{
    let p = embassy_rp::init(Default::default());
    let i2c_i = I2c::new_async(
        p.I2C0,
         p.PIN_5, 
         p.PIN_4, 
         Irqs, 
         Default::default());
    let usb_driver = Driver::new(p.USB, Irqs);
    let _ = spawner.spawn(logger_task(usb_driver).expect("loggernot"));
    let _i = spawner.spawn(mpu_task(i2c_i).expect("imu baglanti hatasi!"));
}