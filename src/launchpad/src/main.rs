// use crate::twin::twin_current_state;
// use crate::twins::mission_twin::{MissionRegistryTwin, MissionTwin};
use crate::twins::{launchpad_twin::LaunchpadTwin, twin};
use actyx_sdk::{app_id, AppManifest, HttpClient};
use futures::StreamExt;
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
    // Http client to connect to actyx
    let service = HttpClient::new(url, app_manifest).await?;

    // let mission_registry = MissionRegistryTwin {};
    // let mission_registry_1_state = twin::execute_twin(service.clone(), mission_registry)?;

    let mut observer = twin::execute_twin(
        service.clone(),
        LaunchpadTwin {
            id: "Launchpad-01".to_string(),
        },
    )
    .unwrap();
    let launchpad = tokio::spawn(async move {
        'observeLaunchpad: loop {
            match observer.next().await {
                Some(state) => println!("launchpad state {:?}", state),
                None => break 'observeLaunchpad,
            }
        }
        println!("I'm done here");
    });

    // let mission_registry = tokio::spawn(launchpad_1_state.map(move |state| {
    //     println!("{:?}", state);
    // }));
    let _ = tokio::join!(launchpad);
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
