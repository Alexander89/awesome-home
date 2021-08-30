use pnet_datalink::interfaces;
use std::time::Duration;
use tokio::time::sleep;
use wifi_rs::{
    prelude::{Config, Connectivity},
    WiFi,
};
use wifiscanner;

pub struct Network();

impl Network {
    pub async fn connect(ssid: String) -> Result<(), anyhow::Error> {
        let adapters = Network::list_adapters();
        println!("available adapters {:?}", adapters);
        let adapter = Network::filter_wifi_adapter(adapters);
        println!("Wifi adapter {:?}", adapter);
        if let Some(adapter) = adapter {
            Network::wait_for_ssid(ssid.clone()).await?;
            Network::connect_wifi(adapter, ssid)?;
            Ok(())
        } else {
            Err(anyhow::Error::msg("no wifi adapter found"))
        }
    }
    #[allow(dead_code)]
    pub fn list_adapters() -> Vec<String> {
        // Get a vector with all network interfaces found
        let all_interfaces = interfaces();

        // Search for the default interface - the one that is
        // up, not loopback and has an IP.
        all_interfaces
            .iter()
            .filter(|e| e.is_up() && !e.is_loopback())
            .map(|interface| interface.name.to_owned())
            .collect()
    }

    #[allow(dead_code)]
    pub fn filter_wifi_adapter(all_interfaces: Vec<String>) -> Option<String> {
        all_interfaces
            .iter()
            .find(|adapter| adapter.to_uppercase().starts_with("WL"))
            .map(|a| a.to_owned())
    }

    #[allow(dead_code)]
    pub async fn wait_for_ssid(ssid: String) -> Result<(), anyhow::Error> {
        let mut retries = 0;
        'scanLoop: loop {
            if let Ok(scan) = wifiscanner::scan() {
                let ssids: Vec<String> = scan.iter().map(|wi| wi.ssid.to_owned()).collect();
                println!("found SSIDs {:?} , scann for '{}'", ssids, ssid);
                if ssids
                    .iter()
                    .any(|w| w.to_uppercase() == ssid.to_uppercase())
                {
                    break 'scanLoop Ok(());
                } else {
                    if retries == 30 {
                        break 'scanLoop Err(anyhow::Error::msg("timed out"));
                    } else {
                        sleep(Duration::from_millis(1000)).await;
                        retries += 1;
                    }
                }
            } else {
                break 'scanLoop Err(anyhow::Error::msg("failed to scan for networks"));
            }
        }
    }

    #[allow(dead_code)]
    pub fn connect_wifi(adapter: String, ssid: String) -> Result<(), anyhow::Error> {
        let config = Some(Config {
            interface: Some(&adapter),
        });

        let mut wifi = WiFi::new(config);

        match wifi.connect(&ssid, "") {
            Ok(_) => {
                println!("connect to drones WIFI {}", ssid);
                Ok(())
            }
            Err(e) => Err(anyhow::Error::msg(format!("wifi error {:?}", e))),
        }
    }
}

#[test]
fn list_adapters() {
    let adapters = Network::list_adapters();

    println!("adapters {:?}", adapters);
    assert!(
        adapters.len() > 0,
        "expect a network card on any computer who compiles rust"
    );
}

#[test]
fn filter_adapters() {
    let adapters =
        Network::filter_wifi_adapter(vec!["enp0s31f6".to_string(), "wlp0s20f3".to_string()]);
    assert_eq!(adapters, Some("sudoi".to_string()));

    let adapters = Network::filter_wifi_adapter(vec!["enp0s31f6".to_string()]);
    assert_eq!(adapters, None);

    let adapters = Network::filter_wifi_adapter(vec![]);
    assert_eq!(adapters, None);
}

#[tokio::test]
async fn wait_for_ssid() {
    let adapters = Network::wait_for_ssid("TELLO-59FF95".to_string()).await;
    println!("res {:?}", adapters);
}
