use crate::drone_control::DroneControl;
#[cfg(feature = "wifi")]
use crate::network::Network;
use crate::twin::{self, resolve_relation};
use crate::twins::drone_twin::states::{DroneTwinState, LaunchedState, ReadyState};
use crate::twins::drone_twin::DroneTwin;
use crate::twins::launchpad_twin::LaunchpadTwinState;
use crate::twins::mission_twin::MissionTwinState;
use crate::twins::{launchpad_twin::LaunchpadTwin, mission_twin::MissionTwin};
use actyx_sdk::HttpClient;
use futures::Stream;
use std::borrow::Borrow;
use std::time::Duration;
use tello::odometry::Odometry;
use tokio::select;
use tokio::time::sleep;
use tokio_stream::StreamExt;

#[cfg(feature = "hardware")]
mod launchpad;
#[cfg(feature = "hardware")]
use crate::launchpad::enable_drone;

pub struct Controller {
    name: String,
    service: HttpClient,
    drone: DroneControl,
}

impl Controller {
    pub fn new(name: String, service: HttpClient) -> Self {
        Self {
            name,
            service,
            drone: DroneControl::new(),
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
            s.mission.map(|id| MissionTwin { id })
        });
        let assigned_drone = resolve_relation(self.service(), launchpad_twin.clone(), |s| {
            s.mounted_drone.map(|id| DroneTwin { id })
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

        loop {
            select! {
                new_launchpad = launchpad_stream.next() => launchpad_state = new_launchpad,
                new_drone = assigned_drone.next() => drone_state = new_drone,
                new_mission = current_mission.next() => mission_state = new_mission,
            }

            println!("\n----------------------------------------");
            println!("launchpad {:?}", launchpad_state);
            println!("drone state {:?}", drone_state);
            println!("current mission {:?}", mission_state);
            println!("--------");

            if let (Some(launchpad), Some(mission)) =
                (launchpad_state.borrow(), mission_state.borrow())
            {
                match (launchpad.drone_enabled, drone_state.borrow()) {
                    (false, Some(drone_state)) => {
                        println!("enable drone {:?}", drone_state);
                        #[cfg(feature = "hardware")]
                        enable_drone().await;
                        #[cfg(not(feature = "hardware"))]
                        sleep(Duration::from_millis(5000)).await;

                        LaunchpadTwin::emit_drone_activated(
                            self.service(),
                            self.name(),
                            drone_state.id(),
                        )
                        .await?;
                    }
                    (true, Some(drone_state)) => match drone_state {
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
                                self.connect_now(id.clone(), ssid.clone(), ip.clone())
                                    .await?
                            } else {
                                self.take_off_now(
                                    id.clone(),
                                    ssid.clone(),
                                    ip.clone(),
                                    mission.id.to_owned(),
                                )
                                .await?
                            }
                        }
                        DroneTwinState::Launched(LaunchedState {
                            id,
                            at_waypoint_id,
                            target_waypoint_id: None,
                            completed: false,
                            ..
                        }) => {
                            self.exec_waypoint(id.to_owned(), *at_waypoint_id as usize, &mission)
                                .await?;
                        }
                        DroneTwinState::Launched(LaunchedState {
                            id,
                            completed: true,
                            ..
                        }) => self.land_now(id.clone()).await?,
                        DroneTwinState::Launched(_) => {
                            println!("wait for next task")
                        }
                        DroneTwinState::Used(_) => {
                            println!("FU do nothing when already in used state")
                        }
                    },
                    _ => (),
                }
            }
            println!("----------------------------------------\n");
        }
    }
}

impl Controller {
    async fn connect_now(
        &mut self,
        id: String,
        ssid: String,
        ip: String,
    ) -> Result<(), anyhow::Error> {
        println!("connect to drone now {} {} {}", id, ssid, ip);
        #[cfg(feature = "wifi")]
        {
            println!("activate drone {}", id);
            Network::connect(ssid).await?;
        }

        let ip = if !ip.contains(':') {
            format!("{}:8889", ip)
        } else {
            ip
        };

        match self.drone.connect(ip).await {
            Ok(()) => {
                DroneTwin::emit_drone_connected(self.service(), id)
                    .await
                    .map(|_| ())?;
            }
            Err(e) => {
                println!("failed to connect to drone {}", e);
                sleep(Duration::from_millis(5000)).await;
            }
        }
        Ok(())
    }

    async fn take_off_now(
        &mut self,
        id: String,
        ssid: String,
        ip: String,
        mission_id: String,
    ) -> Result<(), anyhow::Error> {
        println!("connect drone {}", id);
        if self.drone.is_drone_connected() == false {
            self.connect_now(id.to_owned(), ssid, ip).await?;
        }

        match self.drone.take_off().await {
            Ok(_) => {
                LaunchpadTwin::emit_drone_started(
                    self.service(),
                    "Launchpad-01".to_string(),
                    id.to_owned(),
                    mission_id,
                )
                .await?;
            }
            Err(e) => {
                println!("failed to start drone {}", e);
                sleep(Duration::from_millis(5000)).await;
            }
        }
        Ok(())
    }

    async fn exec_waypoint(
        &mut self,
        drone_id: String,
        current_wp_id: usize,
        mission: &MissionTwinState,
    ) -> Result<(), anyhow::Error> {
        let next_wp = current_wp_id + 1;
        if let Some(wp) = mission.waypoints.get(next_wp) {
            println!("exec waypoint {}", next_wp);
            self.drone
                .exec_waypoint(self.service(), drone_id, mission.id.clone(), wp)
                .await
        } else {
            DroneTwin::emit_drone_mission_completed(self.service(), drone_id, mission.id.clone())
                .await
                .map(|_| ())
        }
    }

    async fn land_now(&mut self, id: String) -> Result<(), anyhow::Error> {
        println!("land drone {}", id);
        match self.drone.land().await {
            Ok(_) => {
                let Odometry { x, y, z, .. } = self.drone.pos();
                DroneTwin::emit_drone_landed(
                    self.service(),
                    "Launchpad-01".to_string(),
                    x as f32,
                    y as f32,
                    z as f32,
                )
                .await?;
            }
            Err(e) => {
                println!("failed to land drone {}", e);
                sleep(Duration::from_millis(5000)).await;
            }
        }
        Ok(())
    }
}
