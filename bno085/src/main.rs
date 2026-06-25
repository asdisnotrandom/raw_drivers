#![no_std]
#![no_main]

use core::f32::consts::PI;
use libm::{atan2f, asinf};
use embassy_rp::gpio::Input;
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

fn quat_to_euler(r: f32, i: f32, j: f32, k: f32) -> (f32, f32, f32)
{
//roll
let sinr_cosp = 2.0 * (r * i + j * k);
let cosr_cosp = 1.0 - 2.0 * (i * i + j * j);
let roll = atan2f(sinr_cosp, cosr_cosp);
//pitch
let sinp = 2.0 * (r * j - k * i);
let pitch = asinf(sinp);
//yaw
let siny_cosp = 2.0 * (r * k + i * j);
let cosy_cosp = 1.0 - 2.0 * (j * j + k * k);
let yaw = atan2f(siny_cosp, cosy_cosp);
//derece
let to_deg = 180.0 / PI;
(roll * to_deg, pitch * to_deg, yaw * to_deg)
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>)
{
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn imu_task(i2c_bus: I2c<'static, embassy_rp::peripherals::I2C0, Async>, hint_pin: embassy_rp::gpio::Input<'static>)
{
    Timer::after_secs(5).await;
    let mut sensor = bno085::Bno085Imu::new(i2c_bus, hint_pin, 0x4A);
    match sensor.init().await
    {
        Ok(_) => log::info!("BNO085 basladi"),
        Err(e) => 
        {
            log::error!("BNO085 baslamadi: {:?}", e);
            return;
        }
    }
    loop {
        match sensor.read_quat().await {
            Ok((r, i, j, k)) => {
                let (roll_val, pitch_val, yaw_val) = quat_to_euler(r, i, j, k);
                log::info!("Roll: {:>6.1} | Pitch: {:>6.1} | Yaw: {:>6.1}", roll_val, pitch_val, yaw_val);
            }
            Err(e) => {

                log::error!("Okuma atlandi. HATA : {:?}", e);
                Timer::after_millis(100).await;
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner)
{
    let p = embassy_rp::init(Default::default());
    let usb_driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(usb_driver).expect("Logger baslamadi"));
    let i2c = I2c::new_async(
        p.I2C0,
        p.PIN_5,
        p.PIN_4,
        Irqs,
        Default::default(),
    );
    let hint = Input::new(p.PIN_16, embassy_rp::gpio::Pull::Up);
    spawner.spawn(imu_task(i2c, hint).expect("imu task baslamadi!"));
}
