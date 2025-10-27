use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_graphics::{image::{Image, ImageRaw}, pixelcolor::BinaryColor, prelude::Point};
use esp_hal::{Async, i2c::master::I2c};
use ssd1306::{
    I2CDisplayInterface, Ssd1306Async, mode::DisplayConfigAsync, prelude::DisplayRotation,
    size::DisplaySize128x64,
};
use embedded_graphics::Drawable;

#[embassy_executor::task]
async fn display_task(mut display: Ssd1306Async<ssd1306::prelude::I2CInterface<I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>>, DisplaySize128x64, ssd1306::mode::BufferedGraphicsModeAsync<DisplaySize128x64>>) {

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);

    let im = Image::new(&raw, Point::new(32, 0));

    im.draw(&mut display).unwrap();

    let _ = display.flush().await;
}

pub async fn start_display(
    spawner: Spawner,
    i2c_dev: I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>,
) -> Result<(), &'static str> {
    let display = init_display(i2c_dev).await?;
    spawner
        .spawn(display_task(display))
        .map_err(|_| "Faild to spawn sense task!")
}

async fn init_display(
    i2c_dev: I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>,
) -> Result<
    Ssd1306Async<
        ssd1306::prelude::I2CInterface<I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>>,
        DisplaySize128x64,
        ssd1306::mode::BufferedGraphicsModeAsync<DisplaySize128x64>,
    >,
    &'static str,
> {
    let interface = I2CDisplayInterface::new(i2c_dev);
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    if let Ok(_) = display.init().await {
        return Ok(display);
    }
    Err("Failed to init ens160")
}
