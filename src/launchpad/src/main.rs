use crate::twin::{resolve_registry, resolve_relation};
use crate::twins::mission_twin::MissionRegistryTwin;
use crate::twins::{launchpad_twin::LaunchpadTwin, mission_twin::MissionTwin};
use actyx_sdk::{app_id, AppManifest, HttpClient};
use tokio::select;
use tokio_stream::StreamExt;
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

    // let launchpad_thread = observe(
    //     twin::execute_twin(
    //         service.clone(),
    //         LaunchpadTwin {
    //             id: "Launchpad-01".to_string(),
    //         },
    //     ),
    //     |state| println!("launchpad state {:?}", state),
    // );
    let mut launchpad_state = twin::execute_twin(
        service.clone(),
        LaunchpadTwin {
            id: "Launchpad-01".to_string(),
        },
    )
    .as_stream();

    let mut all_missions = resolve_registry(service.clone(), MissionRegistryTwin, |s| {
        s.into_iter().map(|id| MissionTwin { id }).collect()
    });

    let mut current_mission = resolve_relation(
        service.clone(),
        LaunchpadTwin {
            id: "Launchpad-01".to_string(),
        },
        |s| s.mission.map(|id| MissionTwin { id }),
    );

    loop {
        select! {
            launchpad = launchpad_state.next() => {
                if let Some(next) = launchpad {
                    println!("current mission {:?}", next)
                }
            },
            mission = all_missions.next() => {
                if let Some(m) = mission {
                    println!("current mission {:?}", m)
                }
            }
            mission = current_mission.next() => {
                println!("current mission {:?}", mission)
            }
        }
    }
}
