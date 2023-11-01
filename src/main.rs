use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use std::{thread::sleep, time::Duration};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Setup peripherals
    let peripherals = Peripherals::take().unwrap();
    let sys_loop: esp_idf_svc::eventloop::EspEventLoop<esp_idf_svc::eventloop::System> =
        EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    info!("Hello, Théo!");

    // Get config with wifi credentials
    let app_config: Config = CONFIG;

    // Connect to wifi using credentials and peripherals
    let _wifi_driver = wifi_connect(peripherals, sys_loop, nvs, app_config);

    loop {
        sleep(Duration::new(10, 0));
    }
}

fn wifi_connect(
    peripherals: Peripherals,
    sys_loop: esp_idf_svc::eventloop::EspEventLoop<esp_idf_svc::eventloop::System>,
    nvs: esp_idf_svc::nvs::EspNvsPartition<esp_idf_svc::nvs::NvsDefault>,
    app_config: Config,
) -> EspWifi<'static> {
    let mut wifi_driver: EspWifi<'_> =
        EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    wifi_driver
        .set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: app_config.wifi_ssid.into(),
            password: app_config.wifi_psk.into(),
            ..Default::default()
        }))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();

    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }
    info!("Should be connected now");

    //log IP info
    info!(
        "IP info: {:?}",
        wifi_driver.sta_netif().get_ip_info().unwrap()
    );

    wifi_driver
}
