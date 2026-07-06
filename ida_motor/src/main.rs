#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::pwm::{Config as PWMConfig, Pwm};
use embassy_rp::Peri;
use panic_probe as _;
use embassy_rp::peripherals::{PIN_0, PIN_2, PIN_4, PIN_6, PWM_SLICE0, PWM_SLICE1, PWM_SLICE2, PWM_SLICE3, USB};
use embassy_rp::usb::{InterruptHandler, Driver};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Config as USBConfig, Builder, UsbDevice};
use embassy_rp::bind_interrupts;
use ida_motor::MotorData;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs
{
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn usb_task(mut dev: UsbDevice<'static, Driver<'static, USB>>)
{
    dev.run().await;
}

#[embassy_executor::task]
async fn motor_task(
    mut class: CdcAcmClass<'static, Driver<'static, USB>>,
    slice0: Peri<'static, PWM_SLICE0>,
    slice1: Peri<'static, PWM_SLICE1>,
    slice2: Peri<'static, PWM_SLICE2>,
    slice3: Peri<'static, PWM_SLICE3>,
    pin0: Peri<'static, PIN_0>,
    pin2: Peri<'static, PIN_2>,
    pin4: Peri<'static, PIN_4>,
    pin6: Peri<'static, PIN_6>,
)
{
    let mut pwmconf = PWMConfig::default();
    let clock_frek = embassy_rp::clocks::clk_sys_freq();
    let calisma_frek = 10_000;
    let divider = 16u8;
    let period = (clock_frek / (calisma_frek * divider as u32)) as u16 - 1;

    pwmconf.top = period;
    pwmconf.divider = divider.into();
    let pwm1 = Pwm::new_output_a(slice0, pin0, pwmconf.clone());
    let pwm2 = Pwm::new_output_a(slice1, pin2, pwmconf.clone());
    let pwm3 = Pwm::new_output_a(slice2, pin4, pwmconf.clone());
    let pwm4 = Pwm::new_output_a(slice3, pin6, pwmconf.clone());
    let mut m1 = MotorData::new(pwm1, period);
    let mut m2 = MotorData::new(pwm2, period);
    let mut m3 = MotorData::new(pwm3, period);
    let mut m4 = MotorData::new(pwm4, period);
    loop
    {
        let _ = m1.set_speed(0.0);
        let _ = m2.set_speed(0.0);
        let _ = m3.set_speed(0.0);
        let _ = m4.set_speed(0.0);

        log::info!("USB bekleniyor");
        class.wait_connection().await;
        log::info!("USB baglandi");
        loop
        {
            let mut buf = [0u8; 16];
            
            match embassy_time::with_timeout(embassy_time::Duration::from_millis(500), class.read_packet(&mut buf)).await {
                Ok(Ok(len)) if len == 16 => {
                    let m1_val = f32::from_le_bytes(buf[0..4].try_into().unwrap());
                    let m2_val = f32::from_le_bytes(buf[4..8].try_into().unwrap());
                    let m3_val = f32::from_le_bytes(buf[8..12].try_into().unwrap());
                    let m4_val = f32::from_le_bytes(buf[12..16].try_into().unwrap());
                    let _ = m1.set_speed(m1_val);
                    let _ = m2.set_speed(m2_val);
                    let _ = m3.set_speed(m3_val);
                    let _ = m4.set_speed(m4_val);
                }
                Ok(Ok(_)) => log::warn!("paket boyutu yanlis"),
                Ok(Err(_)) => break,
                Err(_) => {
                    log::warn!("usb hatasi, motorlar durduruluyor");
                    let _ = m1.set_speed(0.0);
                    let _ = m2.set_speed(0.0);
                    let _ = m3.set_speed(0.0);
                    let _ = m4.set_speed(0.0);
                }
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner)
{
    let p = embassy_rp::init(Default::default());
    let usb_driver = Driver::new(p.USB, Irqs);
    let usb_conf = USBConfig::new(0x1029, 0x0001);
    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    static STATE: StaticCell<State> = StaticCell::new();
    let config_descriptor = CONFIG_DESCRIPTOR.init([0; 256]);
    let bos_descriptor = BOS_DESCRIPTOR.init([0; 256]);
    let control_buf = CONTROL_BUF.init([0; 64]);
    let usb_state = STATE.init(State::new());
    let mut usb_builder = Builder::new(usb_driver, usb_conf, config_descriptor, bos_descriptor, &mut [], control_buf);
    let usb_baba = CdcAcmClass::new(&mut usb_builder, usb_state, 64);
    let usb_device = usb_builder.build();

    spawner.spawn(usb_task(usb_device).unwrap());
    spawner.spawn(motor_task(usb_baba, p.PWM_SLICE0, p.PWM_SLICE1, p.PWM_SLICE2, p.PWM_SLICE3, p.PIN_0, p.PIN_2, p.PIN_4, p.PIN_6).unwrap());
}