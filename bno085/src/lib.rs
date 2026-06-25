#![no_std]

use embedded_hal_async::digital::Wait;
use embedded_hal_async::i2c::I2c;
use core::result::Result::{Err,Ok};
use core::derive;
use core::fmt::Debug;
#[derive(Debug)]
pub enum BnoError<I2cE>
{
    I2c(I2cE),
    InvalidLength(u16),
    WrongChannel(u8),
    UnexpectedReport(u8),
    PinError,
}

pub struct Bno085Imu<I2C, HINT>
{
    i2c: I2C,
    hint_pin: HINT,
    address: u8,
}

impl<I2C, HINT, I2cE> Bno085Imu<I2C, HINT>
where
    I2C: I2c<Error = I2cE>,
    HINT: Wait,
{
    pub fn new(i2c: I2C, hint_pin: HINT, address: u8) -> Self
    {
        Self { i2c, hint_pin, address }
    }
    pub async fn init(&mut self) -> Result<(), BnoError<I2cE>>
    {
        let mut packet = [0u8; 21];
        packet[0] = 21;    // LSB uzunluğu
        packet[1] = 0;     // MSB uzunluğu
        packet[2] = 2;     // Kanal numarası 2 numara kontrol 3 veri okuma
        packet[3] = 0;
        packet[4] = 0xFD;  // report id ; 0xFD = set feature commandı
        packet[5] = 0x05;  // feature ; 0x05 = rotation vector ve quaternion
        packet[6] = 0;
        packet[7] = 0;
        packet[8] = 0;
        packet[9] = 0x20;
        packet[10] = 0x4E; 
        packet[11] = 0x00; 
        packet[12] = 0x00;
        packet[13] = 0;
        packet[14] = 0;
        packet[15] = 0;
        packet[16] = 0;
        packet[17] = 0;
        packet[18] = 0;
        packet[19] = 0;
        packet[20] = 0;
        self.i2c.write(self.address, &packet).await.map_err(BnoError::I2c)?;
        Ok(())
    }
    pub async fn read_quat(&mut self) -> Result<(f32,f32,f32,f32), BnoError<I2cE>>
    {
        self.hint_pin.wait_for_low().await.map_err(|_| BnoError::PinError)?;
        let mut header_check = [0u8; 4];
        self.i2c.read(self.address, &mut header_check).await.map_err(BnoError::I2c)?;
        let bucket_lenght = ((header_check[1] as u16) << 8 | (header_check[0] as u16)) & 0x7FFF;
        let channel = header_check[2];
        if bucket_lenght <= 4
        {
            return Err(BnoError::InvalidLength(bucket_lenght));
        }
        let mut buffer = [0u8; 300];
        let read_len = bucket_lenght as usize;
        if read_len > buffer.len()
        {
            return Err(BnoError::InvalidLength(bucket_lenght));
        }
        self.i2c.read(self.address, &mut buffer[..read_len]).await.map_err(BnoError::I2c)?;
        if channel == 3 
        {
            let index = 9; 
            while index + 14 <= read_len
            {
                let report_id = buffer[index];
                if report_id == 0x05
                { 
                    let i = i16::from_le_bytes([buffer[index + 4], buffer[index + 5]]);
                    let j = i16::from_le_bytes([buffer[index + 6], buffer[index + 7]]);
                    let k = i16::from_le_bytes([buffer[index + 8], buffer[index + 9]]);
                    let r = i16::from_le_bytes([buffer[index + 10], buffer[index + 11]]);
                    const Q_VAL_DTSHT: f32 = 16384.0; //2^14
                    let q_i = (i as f32) / Q_VAL_DTSHT;
                    let q_j = (j as f32) / Q_VAL_DTSHT;
                    let q_k = (k as f32) / Q_VAL_DTSHT;
                    let q_r = (r as f32) / Q_VAL_DTSHT;
                    return Ok((q_r, q_i, q_j, q_k));
                }
                else {return Err(BnoError::UnexpectedReport(report_id));}
            }
        }
        //else {return Err(BnoError::WrongChannel(channel));}
        Err(BnoError::InvalidLength(bucket_lenght))
    }
}