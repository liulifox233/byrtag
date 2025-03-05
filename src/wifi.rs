use anyhow::bail;
use anyhow::Result;
use esp_idf_svc::eventloop::EspEventLoop;
use esp_idf_svc::eventloop::System;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::{
    hal::delay,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration},
};

const SSID: &str = "BUPT-portal";
const PASSWORD: &str = "";

pub fn connect(modem: Modem, sysloop: EspEventLoop<System>) -> Result<Box<EspWifi<'static>>> {
    let nvs = EspDefaultNvsPartition::take()?;

    let mut esp_wifi = esp_idf_svc::wifi::EspWifi::new(modem, sysloop.clone(), Some(nvs))?;

    #[cfg(feature = "random_mac")]
    {
        use esp_idf_svc::wifi::WifiDeviceId;
        let mac = generate_random_mac();
        log::info!("Generated random MAC: {:02X?}", mac);
        esp_wifi.set_mac(WifiDeviceId::Sta, mac)?;
        log::info!(
            "Set MAC address to {:02X?}",
            esp_wifi.get_mac(WifiDeviceId::Sta)?
        );
    }

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: heapless::String::<32>::from_iter(SSID.chars()),
        password: heapless::String::<64>::from_iter(PASSWORD.chars()),
        channel: None,
        auth_method: AuthMethod::None,
        ..Default::default()
    }))?;

    log::info!("Starting wifi...");
    wifi.start()?;
    log::info!("Connecting wifi {}...", SSID);
    let delay: delay::Delay = Default::default();

    for retry in 0..10 {
        match wifi.connect() {
            Ok(_) => break,
            Err(e) => {
                log::warn!(
                    "Failed to connect wifi: {}, will retry after 10 seconds...",
                    e
                );
            }
        }
        delay.delay_ms(1000 * 10);
        if retry == 9 {
            log::error!("Retry limit exceeded");
            bail!("Failed to connect to wifi");
        } else {
            log::info!("Retrying...");
        }
    }

    log::info!("Waiting for DHCP lease...");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("Wifi DHCP info: {:?}", ip_info);

    Ok(Box::new(esp_wifi))
}

#[cfg(feature = "random_mac")]
pub fn generate_random_mac() -> [u8; 6] {
    use rand::Rng;
    let mut rng = rand::rng();
    let mut mac = [0u8; 6];
    rng.fill(&mut mac);
    mac[0] &= 0xFE;
    mac[0] &= 0xFD;
    mac
}
