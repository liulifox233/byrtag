use anyhow::{bail, Result};
use core::fmt;
use embedded_svc::http::{client::Client, Method};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection, FollowRedirectsPolicy};
use std::fmt::Display;

pub const CHECK_URL: &str = "http://connect.rom.miui.com/generate_204?cmd=redirect&arubalp=12345";

pub struct Ap {
    name: String,
    group: String,
    mac: String,
    switch_ip: String,
}

impl Display for Ap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ name: {}, group: {}, mac: {}, switch_ip: {} }}",
            self.name, self.group, self.mac, self.switch_ip
        )
    }
}

pub fn get_ap(url: impl AsRef<str>) -> Result<Ap> {
    log::info!("get ap info with url: {}", url.as_ref());
    let connection = EspHttpConnection::new(&Configuration {
        follow_redirects_policy: FollowRedirectsPolicy::FollowNone,
        ..Default::default()
    })?;
    let mut client = Client::wrap(connection);
    let request = client.request(Method::Get, url.as_ref(), &[])?;
    let response = request.submit()?;
    log::info!("response status: {}", response.status());
    match response.status() {
        // Redirect to login page
        302 => {
            let location = response
                .header("Location")
                .ok_or_else(|| anyhow::anyhow!("no Location header found in response"))?;
            log::info!("redirected to: {}", location);

            let ap = Ap::from(location);

            Ok(ap)
        }
        // Logged in, not redirected
        status => bail!("unexpected status code: {status}"),
    }
}

fn extract_query_param(url: &str, key: &str) -> Result<String> {
    let url = url.as_bytes();
    let key = key.as_bytes();
    let key = format!("{}=", String::from_utf8_lossy(key));
    let key = key.as_bytes();
    let start = url
        .windows(key.len())
        .position(|window| window == key)
        .ok_or_else(|| anyhow::anyhow!("key not found in url"))?;
    let start = start + key.len();
    let end = url[start..]
        .iter()
        .position(|&c| c == b'&')
        .unwrap_or(url.len() - start);
    Ok(std::str::from_utf8(&url[start..start + end])?.to_owned())
}

impl From<&str> for Ap {
    fn from(location: &str) -> Self {
        let name = extract_query_param(location, "name").unwrap_or_default();
        let group = extract_query_param(location, "group").unwrap_or_default();
        let mac = extract_query_param(location, "mac").unwrap_or_default();
        let switch_ip = extract_query_param(location, "switch_ip").unwrap_or_default();
        Ap {
            name,
            group,
            mac,
            switch_ip,
        }
    }
}
