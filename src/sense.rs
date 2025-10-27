
use embassy_embedded_hal::shared_bus::asynch::i2c::{self, I2cDevice};
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    watch::{DynReceiver, Watch},
};
use embassy_time::{Delay, Duration, Timer};
use ens160_aq::Ens160;
use esp_hal::{
    Async, i2c::master::I2c
};
use log::{error, info};


const CO2_CONSUMERS: usize = 1;
static CO2: Watch<CriticalSectionRawMutex, u16, CO2_CONSUMERS> = Watch::new();

pub fn get_receiver() -> Option<DynReceiver<'static, u16>> {
    CO2.dyn_receiver()
}

#[embassy_executor::task]
async fn sense_task(mut ens160: Ens160<I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>, Delay>){

     loop {
        if let Ok(status) = ens160.get_status().await {
            if status.new_data_ready() {
                if let Ok(measurements) = ens160.get_measurements().await{
                    info!("Measurements: {:#?}", measurements);
                } else {
                    info!("Failed to read measurements.");
                }
            }
        } else {
            info!("Failed to get ENS160 status.");
        }
        Timer::after(Duration::from_secs(2)).await;
    } 
}

pub async fn start_sense(spawner: Spawner, i2c_dev: I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>) -> Result<(), &'static str> {
    let ens160 = init_ens(i2c_dev).await?;
    spawner.spawn(sense_task(ens160)).map_err(| _ | {"Faild to spawn sense task!"})
}

async fn init_ens(i2c_dev: I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>) -> Result<Ens160<I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>, Delay>, &'static str>{
    let mut ens160 = Ens160::new(i2c_dev, Delay);
    if let Ok( success) =  ens160.initialize().await {
        if success {return Ok(ens160)}
    }
    let released_dev = ens160.release();
    let mut ens160_alt = Ens160::new_secondary_address(released_dev, Delay);
    if let Ok( success) =  ens160_alt.initialize().await {
        if success {return Ok(ens160_alt)}
    }
    ens160_alt.release();
    return Err("Faild to init ens160");
}

//async fn init_aht(i2c_dev: I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>) -> Result<Aht2X<I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>, Delay>, &'static str>{

