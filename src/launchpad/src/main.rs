use crate::twin::observe_registry;
use crate::twins::mission_twin::MissionRegistryTwin;
use crate::twins::{
    launchpad_twin::LaunchpadTwin, mission_twin::MissionTwin, switch_map::switch_map,
};
use actyx_sdk::{app_id, AppManifest, HttpClient};
use std::thread::sleep;
use std::time::Duration;
use tokio_stream::StreamExt;
use twin::observe;
use url::Url;

mod launchpad;
mod twin;
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
    // Http client to connect to actyx
    let url = Url::parse("http://localhost:4454")?;
    let service = HttpClient::new(url, app_manifest).await?;

    let launchpad_thread = observe(
        twin::execute_twin(
            service.clone(),
            LaunchpadTwin {
                id: "Launchpad-01".to_string(),
            },
        ),
        |state| println!("launchpad state {:?}", state),
    );

    let mut current_mission = switch_map(
        twin::execute_twin(
            service.clone(),
            LaunchpadTwin {
                id: "Launchpad-01".to_string(),
            },
        ),
        |state| {
            state
                .mission
                .and_then(|id| Some(twin::execute_twin(service.clone(), MissionTwin { id })))
        },
    );

    let o = observe_registry(
        service.clone(),
        LaunchpadTwin {
            id: "Launchpad-01".to_string(),
        },
        |s| {
            s.mission
                .and_then(|id| Some(vec![MissionTwin { id }]))
                .unwrap_or_default()
        },
        |s| println!("state: {:?}", s),
    );

    let o = observe_registry(
        service.clone(),
        MissionRegistryTwin,
        |s| s.into_iter().map(|id| MissionTwin { id }).collect(),
        |s| println!("All missions: {:?}", s),
    );

    let res = current_mission.next().await;
    println!("All missions {:?}", res);

    sleep(Duration::from_secs_f32(2.0));

    launchpad_thread.cancel_blocking().await;

    Ok(())
}
