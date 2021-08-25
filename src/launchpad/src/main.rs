use crate::twin::current_state;
use crate::twins::combine_latest::combine_latest;
use crate::twins::mission_twin::MissionRegistryTwin;
use crate::twins::mission_twin::MissionTwin;
use crate::twins::switch_map::switch_map;
use crate::twins::{
    launchpad_twin::LaunchpadTwin,
    twin::{self, observe},
};
use actyx_sdk::{app_id, AppManifest, HttpClient};
use tokio_stream::StreamExt;
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
    // Http client to connect to actyx
    let url = Url::parse("http://localhost:4454")?;
    let service = HttpClient::new(url, app_manifest).await?;
    let missions_thread = observe(
        twin::execute_twin(service.clone(), MissionRegistryTwin),
        |state| println!("Missions state {:?}", state),
    );

    let launchpad_thread = observe(
        twin::execute_twin(
            service.clone(),
            LaunchpadTwin {
                id: "Launchpad-01".to_string(),
            },
        ),
        |state| println!("launchpad state {:?}", state),
    );

    let missions_thread = observe(
        twin::execute_twin(service.clone(), MissionRegistryTwin),
        |state| println!("Missions state {:?}", state),
    );

    let mut s = switch_map(
        twin::execute_twin(service.clone(), MissionRegistryTwin),
        |state| {
            combine_latest(
                state
                    .into_iter()
                    .map(|id| twin::execute_twin(service.clone(), MissionTwin { id }))
                    .collect(),
            )
        },
    );
    let res = s.next().await.unwrap();
    println!("MissionTwinState {:?}", res);

    let _ = tokio::join!(missions_thread, launchpad_thread);
    // let mission_registry = MissionRegistryTwin {};
    // let mission_registry_1_state = twin::execute_twin(service.clone(), mission_registry)?;

    // let mission_registry = tokio::spawn(launchpad_1_state.map(move |state| {
    //     println!("{:?}", state);
    // }));

    // match launchpad_1_state
    //     .try_poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
    //     .await
    // {
    //     Ok(state) => {
    //         println!("{:?}", state);
    //         if let Some(mission) = state.mission {
    //             println!("{:?}", mission);
    //             match *(twin_current_state(service.clone(), MissionTwin { id: mission }).await)
    //             {
    //                 Ok(mission_state) => println!("{:?}", mission_state),
    //                 Err(e) => println!("{:?}", e),
    //             }
    //         }
    //     }
    //     _ => (),
    // }
    // match mission_registry_1_state.try_recv() {
    //     Ok(state) => {
    //         println!("{:?}", state);
    //     }
    //     _ => (),
    // }
    // std::thread::sleep(Duration::from_secs_f32(0.33f32));
    Ok(())
}
