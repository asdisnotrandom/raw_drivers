#![no_std]

use embedded_hal::pwm::{SetDutyCycle};

pub struct MotorData<PWM>
{
    pwm_pin: PWM,
    max_duty: u16,
}

impl<PWM, E> MotorData<PWM>
where
    PWM: SetDutyCycle<Error= E>,
{
    pub fn new(pwm_pin: PWM, max_duty: u16) -> Self
    {
        Self {pwm_pin, max_duty}
    }
    pub fn set_speed(&mut self, percentage: f32) -> Result<(), E>
    {
        let percentage = percentage.clamp(0.0, 100.0);
        let duty = ((percentage / 100.0) * self.max_duty as f32) as u16;
        self.pwm_pin.set_duty_cycle(duty)
    }
}