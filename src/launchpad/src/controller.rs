use crate::{
    hardware::Hardware,
    twin::{self, resolve_relation},
    twins::{
        drone_twin::{
            states::{DroneTwinState, LaunchedState, ReadyState},
            DroneTwin,
        },
        launchpad_twin::{LaunchpadTwin, LaunchpadTwinState},
        mission_twin::{MissionTwin, MissionTwinState},
    },
};
use actyx_sdk::HttpClient;
use futures::Stream;
use std::time::Duration;
use tokio::{select, sync::mpsc, time::interval};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tokio_stream_ext::StreamOpsExt;

pub struct Controller {
    name: String,
    service: HttpClient,
    hardware: Hardware,
}
#[derive(Clone, Debug)]
struct AppState {
    pub launchpad: LaunchpadTwinState,
    pub drone: Option<DroneTwinState>,
    pub mission: Option<MissionTwinState>,
}

impl Controller {
    pub fn new(name: String, service: HttpClient) -> Self {
        Self {
            name,
            hardware: Hardware::new(service.clone()),
            service,
        }
    }
    fn service(&self) -> HttpClient {
        self.service.clone()
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        LaunchpadTwin::emit_launchpad_registered(self.service(), self.name()).await?;

        let launchpad_twin = LaunchpadTwin::new(self.name());

        let launchpad_stream =
            twin::execute_twin(self.service(), launchpad_twin.clone()).as_stream();

        let current_mission = resolve_relation(self.service(), launchpad_twin.clone(), |s| {
            s.current_mission.map(|id| MissionTwin { id })
        });
        let assigned_drone = resolve_relation(self.service(), launchpad_twin.clone(), |s| {
            s.attached_drone.map(|id| DroneTwin { id })
        });

        let res = self
            .logic(launchpad_stream, current_mission, assigned_drone)
            .await;
        println!("Program terminated with {:#?}", res);
        Ok(())
    }

    async fn logic(
        &mut self,
        mut launchpad_stream: impl Stream<Item = LaunchpadTwinState> + Unpin,
        mut current_mission: impl Stream<Item = MissionTwinState> + Unpin,
        mut assigned_drone: impl Stream<Item = DroneTwinState> + Unpin,
    ) -> Result<(), anyhow::Error> {
        let mut launchpad_state = None;
        let mut drone_state = None;
        let mut mission_state = None;

        let mut state_read = interval(Duration::from_millis(1000));
        let (tx, rx) = mpsc::channel::<AppState>(3);

        let mut state_update =
            Box::pin(ReceiverStream::new(rx).debounce(Duration::from_millis(200)));

        loop {
            select! {
                _ = state_read.tick() => {
                    drone_state.as_ref().map(|s| self.update_states(s));
                },
                new_launchpad = launchpad_stream.next() => {
                    launchpad_state = new_launchpad;
                    let _ = tx.send(AppState {
                        drone: drone_state.clone(),
                        launchpad: launchpad_state.clone().unwrap(),
                        mission: mission_state.clone(),
                    }).await;
                },
                new_drone = assigned_drone.next() =>{
                    drone_state = new_drone;
                    if let Some(launchpad) = launchpad_state.clone() {
                        let _ = tx.send(AppState {
                            drone: drone_state.clone(),
                            launchpad,
                            mission: mission_state.clone(),
                        }).await;
                    }
                },
                new_mission = current_mission.next() => {
                    mission_state = new_mission;
                    if let Some(launchpad) = launchpad_state.clone() {
                        let _ = tx.send(AppState {
                            drone: drone_state.clone(),
                            launchpad,
                            mission: mission_state.clone(),
                        }).await;
                    }
                },
                Some(app_state) = state_update.next() => {
                    if let Err(e) = self.handler(app_state.clone()).await {
                        println!("Something is wrong {:?}", e);
                    };
                },

            }
        }
    }
    async fn update_states(&mut self, drone_state: &DroneTwinState) -> Result<(), anyhow::Error> {
        println!("update_states");
        if let Ok(s) = self.hardware.get_state() {
            let (battery, id) = match drone_state {
                DroneTwinState::Undefined(_) => return Ok(()),
                DroneTwinState::Ready(e) => (e.battery, e.id.to_owned()),
                DroneTwinState::Launched(e) => (e.battery, e.id.to_owned()),
                DroneTwinState::Used(e) => (e.battery, e.id.to_owned()),
            };

            println!("battery {} <-> new {}", battery, s.bat);

            if (battery as i8 - s.bat).abs() >= 5 {
                println!("update battery values {:?}", s.bat);
                DroneTwin::emit_drone_stats_updated(self.service(), id, s.bat as u8).await?;
            }
        };
        Ok(())
    }

    async fn handler(&mut self, app_state: AppState) -> Result<(), anyhow::Error> {
        let launchpad_state = app_state.launchpad;
        let mission_state = app_state.mission;
        let drone_state = app_state.drone;

        if let (Some(drone_state), Some(mission)) = (drone_state, mission_state) {
            match drone_state {
                DroneTwinState::Undefined(_) => {
                    println!("FU: Starting an undefined drone!? NO!")
                }
                // drone is defined
                DroneTwinState::Ready(ref d @ ReadyState { .. }) if !d.is_enabled() => {
                    println!("enable drone");
                    self.hardware.enable_drone().await;
                    DroneTwin::emit_drone_activated(self.service(), d.id.to_owned(), self.name())
                        .await?;
                }
                // drone is enabled
                DroneTwinState::Ready(ReadyState {
                    id,
                    ssid,
                    ip,
                    connected: false,
                    ..
                }) => {
                    println!("connect to drone now");
                    self.hardware
                        .connect_now(id.clone(), ssid.clone(), ip.clone())
                        .await?
                }
                // drone is enabled / and connected
                DroneTwinState::Ready(ReadyState {
                    id,
                    ssid,
                    ip,
                    connected: true,
                    ..
                }) => {
                    self.hardware
                        .take_off_now(id.clone(), ssid.clone(), ip.clone(), mission.id.to_owned())
                        .await?
                }
                // drone is in the air and the current mission is *not* completed an currently not moving to the next waypoint
                DroneTwinState::Launched(LaunchedState {
                    id,
                    at_waypoint_id,
                    target_waypoint_id: None,
                    completed: false,
                    ..
                }) => {
                    self.hardware
                        .exec_waypoint(id.to_owned(), at_waypoint_id as usize, &mission)
                        .await?;
                }
                // Mission completed land now!
                DroneTwinState::Launched(LaunchedState {
                    id,
                    completed: true,
                    ..
                }) => {
                    self.hardware.land_now(id.clone()).await?;
                    DroneTwin::emit_drone_mission_completed(self.service(), id, mission.id.clone())
                        .await?;
                }
                // drone is on the way to the next waypoint, and wait that the drone arrives on the next Waypoint
                DroneTwinState::Launched(LaunchedState {
                    id,
                    mission_id,
                    target_waypoint_id: Some(next_wp),
                    completed: false,
                    ..
                }) => {
                    println!(
                        "Drone {} is on the way to {} for mission {}",
                        id, next_wp, mission_id
                    );
                }
                DroneTwinState::Used(_) => {
                    println!("FU do nothing when already in used state")
                }
            }
        } else {
            if let Some(next_mission) = launchpad_state.mission_queue.first() {
                println!("Activate next mission {}", next_mission);
                LaunchpadTwin::emit_mission_activated(
                    self.service(),
                    self.name(),
                    next_mission.to_owned(),
                )
                .await?;
            }
        }

        println!("----------------------------------------\n");
        Ok(())
    }
}
