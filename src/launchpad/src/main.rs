use std::borrow::Borrow;
use std::time::Duration;

use crate::drone_control::DroneControl;
use crate::twin::resolve_relation;
use crate::twins::drone_twin::states::{DroneTwinState, LaunchedState, ReadyState};
use crate::twins::drone_twin::DroneTwin;
use crate::twins::{launchpad_twin::LaunchpadTwin, mission_twin::MissionTwin};
use actyx_sdk::service::EventService;
use actyx_sdk::{app_id, AppManifest, HttpClient};
use tokio::select;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use url::Url;

#[cfg(feature = "hardware")]
mod launchpad;
#[cfg(feature = "hardware")]
use crate::launchpad::enable_drone;

mod drone_control;
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
    // Http client to connect to actyx
    let url = Url::parse("http://localhost:4454")?;
    let service = HttpClient::new(url, app_manifest).await?;
    let mut drone = DroneControl::new();

    LaunchpadTwin::emit_launchpad_registered(service.clone(), name.clone()).await?;

    let launchpad_twin = LaunchpadTwin::new(name.clone());

    let mut launchpad_stream =
        twin::execute_twin(service.clone(), launchpad_twin.clone()).as_stream();

    let mut current_mission = resolve_relation(service.clone(), launchpad_twin.clone(), |s| {
        s.mission.map(|id| MissionTwin { id })
    });
    let mut assigned_drone = resolve_relation(service.clone(), launchpad_twin.clone(), |s| {
        s.mounted_drone.map(|id| DroneTwin { id })
    });

    let mut launchpad_state = None;
    let mut drone_state = None;
    let mut mission_state = None;

    loop {
        select! {
            new_launchpad = launchpad_stream.next() => launchpad_state = new_launchpad,
            new_drone = assigned_drone.next() => drone_state = new_drone,
            new_mission = current_mission.next() => mission_state = new_mission,
        }
        if let (Some(launchpad), Some(mission)) = (launchpad_state.borrow(), mission_state.borrow())
        {
            match (launchpad.mounted_drone.borrow(), drone_state.borrow()) {
                (Some(mounted_drone), None) => {
                    println!("enable drone");
                    #[cfg(feature = "hardware")]
                    enable_drone().await;
                    #[cfg(not(feature = "hardware"))]
                    sleep(Duration::from_millis(5000)).await;

                    LaunchpadTwin::emit_drone_activated(
                        service.clone(),
                        name.clone(),
                        mounted_drone.to_owned(),
                    )
                    .await?;
                    //LaunchpadTwin::emit_launchpad_registered(service.clone(), "Launchpad-01".to_string()).await?;
                }
                (Some(_), Some(drone_state)) => {
                    match drone_state {
                        DroneTwinState::Undefined(_) => {
                            println!("FU: Starting an undefined drone!? NO!")
                        }
                        DroneTwinState::Ready(ReadyState {
                            id,
                            ssid,
                            ip,
                            connected,
                            ..
                        }) => {
                            if !connected {
                                connect_now(
                                    service.clone(),
                                    &mut drone,
                                    id.clone(),
                                    ssid.clone(),
                                    ip.clone(),
                                )
                                .await?
                            } else {
                                take_off_now(
                                    service.clone(),
                                    &mut drone,
                                    id.clone(),
                                    mission.id.to_owned(),
                                )
                                .await?
                            }
                        }
                        DroneTwinState::Launched(LaunchedState { id, .. }) => {
                            if let Ok(_) = drone.take_off().await {
                                LaunchpadTwin::emit_drone_started(
                                    service.clone(),
                                    "Launchpad-01".to_string(),
                                    id.to_owned(),
                                    mission.id.to_owned(),
                                )
                                .await?;
                            } else {
                                println!("failed to start drone");
                                sleep(Duration::from_millis(5000)).await;
                            }
                        }
                        DroneTwinState::Used(_) => {
                            println!("FU do nothing when already in used state")
                        }
                    }
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

async fn connect_now(
    service: impl EventService,
    drone: &mut DroneControl,
    id: String,
    ssid: String,
    ip: String,
) -> Result<(), anyhow::Error> {
    println!("connect drone {}", id);
    if let Ok(()) = drone.connect(ssid, ip).await {
        DroneTwin::emit_drone_connected(service.clone(), id)
            .await
            .map(|_| ())?;
    } else {
        println!("failed to connect to drone");
        sleep(Duration::from_millis(5000)).await;
    }
    Ok(())
}

async fn take_off_now(
    service: impl EventService,
    drone: &mut DroneControl,
    id: String,
    mission_id: String,
) -> Result<(), anyhow::Error> {
    println!("connect drone {}", id);
    if let Ok(_) = drone.take_off().await {
        LaunchpadTwin::emit_drone_started(
            service.clone(),
            "Launchpad-01".to_string(),
            id.to_owned(),
            mission_id,
        )
        .await?;
    } else {
        println!("failed to start drone");
        sleep(Duration::from_millis(5000)).await;
    }
    Ok(())
}
