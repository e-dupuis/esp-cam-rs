use embedded_svc::http::{client::Client as HttpClient, Method};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::client::EspHttpConnection,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
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

fn main() -> anyhow::Result<()> {
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

    // Init wifi driver
    let mut wifi_driver = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    // Connect to wifi using credentials and peripherals
    wifi_connect(&mut wifi_driver, &app_config);

    get("http://mayeul.net/");

    loop {
        sleep(Duration::new(10, 0));
    }
}

fn wifi_connect(
    wifi_driver: &mut BlockingWifi<EspWifi<'static>>,
    app_config: &Config,
) -> anyhow::Result<()> {
    wifi_driver
        .set_configuration(&embedded_svc::wifi::Configuration::Client(
            embedded_svc::wifi::ClientConfiguration {
                ssid: app_config.wifi_ssid.into(),
                password: app_config.wifi_psk.into(),
                ..Default::default()
            },
        ))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();

    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }

    wifi_driver.wait_netif_up()?;
    info!("Should be connected now");

    //log IP info
    info!(
        "IP info: {:?}",
        wifi_driver.wifi().sta_netif().get_ip_info().unwrap()
    );
    Ok(())
}

fn get(url: impl AsRef<str>) -> anyhow::Result<()> {
    // 1. Create a new EspHttpConnection with default Configuration. (Check documentation)
    info!("Init connection");
    let connection = esp_idf_svc::http::client::EspHttpConnection::new(
        &esp_idf_svc::http::client::Configuration::default(),
    );
    // 2. Get a client using the Client::wrap method. (Check documentation)
    info!("Get client");
    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

    // 3. Open a GET request to `url`
    info!("Prepare request");
    let headers = [("accept", "text/plain")];
    let request = client.request(Method::Get, url.as_ref(), &headers)?;

    // 4. Submit the request and check the status code of the response.
    info!("-> GET {}", url.as_ref());
    let response = request.submit()?;
    let status = response.status();
    info!("Response code: {}\n", status);
    match status {
        200..=299 => Ok(()),
        default => Err(status),
    };
    // Successful http status codes are in the 200..=299 range.

    // 5. If the status is OK, read response data chunk by chunk into a buffer and print it until done.

    // 6. Try converting the bytes into a Rust (UTF-8) string and print it.
    // }

    Ok(())
}
