#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]


use embassy_embedded_hal::shared_bus::asynch::i2c::{self, I2cDevice};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use esp_rtos;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{Async, i2c::master};
use esp_hal::{clock::CpuClock, i2c::master::Config, time::Rate};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_println::logger::init_logger;
use log::{error, info};
use static_cell::StaticCell;

mod sense;
mod display;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    /************* init ******************/
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);

    let timg0 = TimerGroup::new(p.TIMG0);
    let software_interrupt = SoftwareInterruptControl::new(p.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);
    init_logger(log::LevelFilter::Debug);

    info!("Starting up...");

    static I2C_BUS: StaticCell<Mutex<NoopRawMutex, esp_hal::i2c::master::I2c<Async>>> =
        StaticCell::new();
    let i2c = esp_hal::i2c::master::I2c::new(
        p.I2C0,
        Config::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_sda(p.GPIO22)
    .with_scl(p.GPIO23)
    .into_async();

    let i2c_bus = &*I2C_BUS.init(Mutex::new(i2c));
    let i2c_sensor_device: i2c::I2cDevice<'static, NoopRawMutex, master::I2c<'static, Async>> =
        I2cDevice::new(i2c_bus);

    let i2c_display_device: i2c::I2cDevice<'static, NoopRawMutex, master::I2c<'static, Async>> =
        I2cDevice::new(i2c_bus);

    spawner.spawn(welcome_task()).unwrap();

    let _ = sense::start_sense(spawner, i2c_sensor_device).await;
    let _ = display::start_display(spawner, i2c_display_device).await;
}

#[embassy_executor::task]
async fn welcome_task() {
    info!("Hello =)");
    Timer::after(Duration::from_secs(1)).await;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {
        error!("PANIC: {info}");
    }
}
