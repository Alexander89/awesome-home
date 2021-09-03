use crate::twins::launchpad_twin::LaunchpadTwin;
use actyx_sdk::{app_id, AppManifest, HttpClient};
use url::Url;

mod controller;
mod hardware;
mod twin;
mod twins;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let name = "Launchpad-01".to_string();
    // add your app manifest, for brevity we will use one in trial mode
    let app_manifest = AppManifest::new(
        app_id!("com.example.launchpad"),
        "Drone Launchpad".into(),
        "0.1.0".into(),
        None,
    );

    // Url of the locally running Actyx node
    let url = Url::parse("http://localhost:4454")?;
    // Http client to connect to actyx
    let service = HttpClient::new(url, app_manifest).await?;
    LaunchpadTwin::emit_launchpad_registered(service.clone(), name.clone()).await?;

    controller::Controller::new(name, service).start().await?;

    Ok(())
}
