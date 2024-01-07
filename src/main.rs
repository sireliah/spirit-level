#![no_main]
#![no_std]

use cortex_m::prelude::_embedded_hal_serial_Write;
use cortex_m_rt::entry;
use heapless::Vec;
use lsm303agr::{AccelMode, AccelOutputDataRate, Lsm303agr};
use microbit::{
    hal::prelude::*,
    hal::twim,
    hal::uarte::{self, Baudrate, Error, Instance, Parity},
    hal::Timer,
    pac::twim0::frequency::FREQUENCY_A,
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

mod serial_setup;
use serial_setup::UartePort;

fn write_string<T: Instance>(
    serial: &mut UartePort<T>,
    data: &mut Vec<u8, 256>,
) -> Result<(), Error> {
    for byte in data.into_iter() {
        nb::block!(serial.write(*byte))?;
    }
    nb::block!(serial.flush())?;
    Ok(())
}

fn format_row(x: i32, y: i32, z: i32) -> Vec<u8, 256> {
    let mut str_buff = Vec::<u8, 256>::new();
    str_buff.extend(x.to_be_bytes());
    str_buff.push(',' as u8);
    str_buff.extend(y.to_be_bytes());
    str_buff.push(',' as u8);
    str_buff.extend(z.to_be_bytes());
    str_buff.push('\n' as u8);
    str_buff
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    if let Some(board) = microbit::Board::take() {
        let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };
        let mut timer = Timer::new(board.TIMER0);
        let mut sensor = Lsm303agr::new_with_i2c(i2c);

        timer.delay(1000u32);
        sensor.init().expect("Failed to initialize accelerometer");

        sensor
            .set_accel_mode_and_odr(&mut timer, AccelMode::Normal, AccelOutputDataRate::Hz1)
            .expect("Failed setting accelerometer mode");

        let mut serial = {
            let serial = uarte::Uarte::new(
                board.UARTE0,
                board.uart.into(),
                Parity::EXCLUDED,
                Baudrate::BAUD115200,
            );
            UartePort::new(serial)
        };

        loop {
            match sensor.accel_status() {
                Ok(status) => {
                    if status.xyz_new_data() {
                        match sensor.acceleration() {
                            Ok(acceleration) => {
                                let (x, y, z) = acceleration.xyz_mg();
                                rprintln!("{} {} {}", x, y, z);

                                let mut str_buff = format_row(x, y, z);
                                write_string(&mut serial, &mut str_buff);
                            }
                            Err(err) => {
                                rprintln!("Error: {:?}", err);
                                continue;
                            }
                        };
                    }
                }
                Err(err) => {
                    rprintln!("Error: {:?}", err);
                    continue;
                }
            }
        }
    }
    loop {}
}
