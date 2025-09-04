#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use log::{error, info};
use embassy_time::{Duration, Timer};


#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    spawner.spawn(welcome_task()).unwrap();
    info!("Starting up...");
}

#[embassy_executor::task]
async fn welcome_task() {
    loop {
        info!("Hello =)");
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {
        error!("PANIC: {info}");
    }
}
