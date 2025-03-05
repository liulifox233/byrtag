pub mod bupt;
pub mod wifi;

use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::prelude::*};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;

    let mut esp_wifi = wifi::connect(peripherals.modem, sysloop)?;

    let ap = bupt::get_ap(bupt::CHECK_URL)?;

    log::info!("Got AP: {}", ap);

    esp_wifi.stop()?;

    Ok(())
}
