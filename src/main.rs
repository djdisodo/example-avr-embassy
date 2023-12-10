#![no_std]
#![no_main]
#![feature(type_alias_impl_trait, future_join, abi_avr_interrupt)]
use core::fmt::Write;
use core::panic::PanicInfo;
use arduino_hal::{DefaultClock};
use avr_hal_generic::avr_device;
use avr_tc0_embassy_time::{define_interrupt, init_system_time};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use ufmt::uWrite;

static SERIAL: Mutex<CriticalSectionRawMutex, Option<atmega_hal::usart::Usart0<DefaultClock>>> = Mutex::new(None);
#[embassy_executor::task]
async fn serial_print_a_every1sec() {
    loop {
        embassy_time::Timer::after_secs(1).await;
        SERIAL.lock().await.as_mut().unwrap().write_str("task \"a\" will print every seconds\r\n").unwrap();
    }
}
#[embassy_executor::task]
async fn serial_print_b_every2sec() {
    loop {
        embassy_time::Timer::after_secs(3).await;
        SERIAL.lock().await.as_mut().unwrap().write_str("task \"b\" will print every three seconds\r\n").unwrap();
    }
}

#[embassy_executor::task]
async fn serial_print_c_often() {
    loop {
        embassy_time::Timer::after_millis(200).await;
        SERIAL.lock().await.as_mut().unwrap().write_str("task c will print every 200ms\r\n").unwrap();
    }
}


define_interrupt!(atmega328p);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    *SERIAL.lock().await = Some(arduino_hal::default_serial!(dp, pins, 57600));
    init_system_time(&mut dp.TC0);
    embassy_time::Timer::after_secs(3).await;
    spawner.spawn(serial_print_a_every1sec()).unwrap();
    spawner.spawn(serial_print_b_every2sec()).unwrap();
    spawner.spawn(serial_print_c_often()).unwrap();
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // disable interrupts - firmware has panicked so no ISRs should continue running
    avr_device::interrupt::disable();

    // get the peripherals so we can access serial and the LED.
    //
    // SAFETY: Because main() already has references to the peripherals this is an unsafe
    // operation - but because no other code can run after the panic handler was called,
    // we know it is okay.
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut writer = Writer {
        u_write: serial
    };
    writeln!(&mut writer, "{}", info).ok();
    loop {

    }
}

struct Writer<T> {
    u_write: T
}

impl<T: uWrite> Write for Writer<T> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.u_write.write_str(s).map_err(|x| match x {
            _ => core::fmt::Error
        })
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.u_write.write_char(c).map_err(|x| match x {
            _ => core::fmt::Error
        })
    }
}