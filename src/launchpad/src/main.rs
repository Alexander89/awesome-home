use std::borrow::Borrow;
use std::time::Duration;

use crate::twin::resolve_relation;
use crate::twins::{launchpad_twin::LaunchpadTwin, mission_twin::MissionTwin};
use actyx_sdk::{app_id, AppManifest, HttpClient};
use tokio::select;
use tokio::time::sleep;
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

    LaunchpadTwin::emit_launchpad_registered(service.clone(), "Launchpad-01".to_string()).await?;

    // let launchpad_thread = observe(
    //     twin::execute_twin(
    //         service.clone(),
    //         LaunchpadTwin {
    //             id: "Launchpad-01".to_string(),
    //         },
    //     ),
    //     |state| println!("launchpad state {:?}", state),
    // );
    let mut launchpad_stream = twin::execute_twin(
        service.clone(),
        LaunchpadTwin {
            id: "Launchpad-01".to_string(),
        },
    )
    .as_stream();

    // let mut all_missions = resolve_registry(service.clone(), MissionRegistryTwin, |s| {
    //     s.into_iter().map(|id| MissionTwin { id }).collect()
    // });

    let mut current_mission = resolve_relation(
        service.clone(),
        LaunchpadTwin {
            id: "Launchpad-01".to_string(),
        },
        |s| s.mission.map(|id| MissionTwin { id }),
    );

    let mut launchpad_state = None;
    // let mut missions_state = None;
    let mut mission_state = None;

    loop {
        select! {
            new_launchpad = launchpad_stream.next() => launchpad_state = new_launchpad,
            // new_missions = all_missions.next() => missions_state = new_missions,
            new_mission = current_mission.next() => mission_state = new_mission,
        }
        if let (Some(launchpad), Some(mission)) = (launchpad_state.borrow(), mission_state.borrow())
        {
            match (launchpad.mounted_drone.borrow(), launchpad.drone_enabled) {
                (Some(mounted_drone), false) => {
                    println!("enable drone");
                    sleep(Duration::from_millis(5000)).await;

                    LaunchpadTwin::emit_drone_activated(
                        service.clone(),
                        "Launchpad-01".to_string(),
                        mounted_drone.to_owned(),
                    )
                    .await?;
                    //LaunchpadTwin::emit_launchpad_registered(service.clone(), "Launchpad-01".to_string()).await?;
                }
                (Some(mounted_drone), true) => {
                    println!("drone started");
                    sleep(Duration::from_millis(5000)).await;
                    LaunchpadTwin::emit_drone_started(
                        service.clone(),
                        "Launchpad-01".to_string(),
                        mounted_drone.to_owned(),
                        mission.id.to_owned(),
                    )
                    .await?;
                    //LaunchpadTwin::emit_launchpad_registered(service.clone(), "Launchpad-01".to_string()).await?;
                }
                _ => (),
            }
        }

        println!("--------\n");
        println!("launchpad {:?}", launchpad_state);
        println!("current mission {:?}", mission_state);
        println!("--------\n");
    }
}
