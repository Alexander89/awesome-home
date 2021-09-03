use crate::twins::drone_twin::DroneTwin;
use crate::twins::mission_twin::MissionTwinState;
use actyx_sdk::HttpClient;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
use tello::command_mode::CommandModeState;
use tello::odometry::Odometry;
use tokio::time::sleep;

pub mod drone_control;
use self::drone_control::DroneControl;

#[cfg(feature = "wifi")]
mod network;
#[cfg(feature = "wifi")]
use network::Network;

#[cfg(feature = "hardware")]
mod launchpad;

pub struct Hardware {
    service: HttpClient,
    drone: DroneControl,
}
impl Hardware {
    pub fn new(service: HttpClient) -> Self {
        Self {
            service,
            drone: DroneControl::new(),
        }
    }
    fn service(&self) -> HttpClient {
        self.service.clone()
    }

    pub fn get_state(&mut self) -> Result<CommandModeState, TryRecvError> {
        self.drone.try_recv_state()
    }
}

impl Hardware {
    pub async fn enable_drone(&mut self) {
        #[cfg(feature = "hardware")]
        self::launchpad::enable_drone().await;
        #[cfg(not(feature = "hardware"))]
        sleep(Duration::from_millis(5000)).await;
    }

    pub async fn connect_now(
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

    pub async fn take_off_now(
        &mut self,
        id: String,
        ssid: String,
        ip: String,
        mission_id: String,
    ) -> Result<(), anyhow::Error> {
        println!("take_off drone {}", id);
        if self.drone.is_drone_connected() == false {
            self.connect_now(id.to_owned(), ssid, ip).await?;
        }

        match self.drone.take_off().await {
            Ok(_) => {
                println!("trigger emit_drone_started");
                DroneTwin::emit_drone_launched(self.service(), id.to_owned(), mission_id).await?;
            }
            Err(e) => {
                println!("failed to start drone {}", e);
                sleep(Duration::from_millis(5000)).await;
            }
        }
        Ok(())
    }

    pub async fn exec_waypoint(
        &mut self,
        drone_id: String,
        current_wp_id: usize,
        mission: &MissionTwinState,
    ) -> Result<(), anyhow::Error> {
        let next_wp = current_wp_id + 1;
        if let Some(wp) = mission.waypoints.get(next_wp) {
            println!("exec waypoint {}", next_wp);
            let command_result = self
                .drone
                .exec_waypoint(
                    self.service(),
                    drone_id.clone(),
                    mission.id.clone(),
                    wp,
                    next_wp as i32,
                )
                .await;

            if let Err(e) = command_result {
                println!("command failed {:?}", e);
                sleep(Duration::new(5, 0)).await;
                let _ = self.land_now(drone_id.clone()).await;
                let _ = DroneTwin::emit_drone_mission_completed(
                    self.service(),
                    drone_id,
                    mission.id.clone(),
                )
                .await;
            }
            Ok(())
        } else {
            DroneTwin::emit_drone_mission_completed(self.service(), drone_id, mission.id.clone())
                .await
                .map(|_| ())
        }
    }

    pub async fn land_now(&mut self, id: String) -> Result<(), anyhow::Error> {
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
