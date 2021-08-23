use crate::twins::{launchpad_twin::LaunchpadTwin, twin};
use actyx_sdk::{app_id, AppManifest, HttpClient};
use std::time::Duration;
use url::Url;

mod launchpad;
mod twins;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    // add your app manifest, for brevity we will use one in trial mode
    let app_manifest = AppManifest::new(
        app_id!("com.example.launchpad"),
        "Drone Launchpad".into(),
        "0.1.0".into(),
        None,
    );

    // Url of the locally running Actyx node
    let url = Url::parse("http://localhost:4454")?;
    // client for
    let service = HttpClient::new(url, app_manifest).await?;

    let launchpad_1 = LaunchpadTwin {
        id: "1".to_string(),
    };
    let state_changed = twin::execute_twin(service.clone(), launchpad_1)?;

    loop {
        match state_changed.recv() {
            Ok(state) => {
                println!("{:?}", state);
            }
            Err(e) => {
                println!("err: {}", e);
            }
        }
        std::thread::sleep(Duration::from_secs_f32(0.33f32));
    }
}
