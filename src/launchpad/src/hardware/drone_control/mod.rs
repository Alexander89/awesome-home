use std::{borrow::BorrowMut, sync::mpsc::TryRecvError, time::Duration};

use actyx_sdk::service::EventService;
use tello::{command_mode::CommandModeState, odometry::Odometry, CommandMode, Drone};
use tokio::time::sleep;

use crate::twins::{
    drone_twin::DroneTwin,
    mission_twin::types::{DelayWaypoint, GoToWaypoint, TurnWaypoint, Waypoint},
};

pub struct DroneControl {
    drone: Option<CommandMode>,
}

impl DroneControl {
    pub fn new() -> Self {
        Self { drone: None }
    }

    #[allow(dead_code)]
    pub fn try_recv_state(&mut self) -> Result<CommandModeState, TryRecvError> {
        if let Some(d) = self.drone.borrow_mut() {
            let mut last: Option<CommandModeState> = None;
            while let Ok(s) = d.state_receiver.try_recv() {
                last = Some(s);
            }
            println!("new State {:?}", last);
            last.map(|s| Ok(s)).unwrap_or(Err(TryRecvError::Empty))
        } else {
            Err(TryRecvError::Disconnected)
        }
    }

    pub fn is_drone_connected(&self) -> bool {
        self.drone.as_ref().is_some()
    }

    pub async fn connect(&mut self, ip: String) -> Result<(), String> {
        if let None = self.drone.as_ref() {
            let drone = Drone::new(&*ip).command_mode();
            self.drone = Some(drone);
        }
        self.drone.as_mut().unwrap().enable().await
    }
    pub async fn take_off(&mut self) -> Result<(), String> {
        if let Some(drone) = self.drone.as_mut() {
            drone.take_off().await
        } else {
            Err("no drone connected".to_string())
        }
    }
    pub async fn exec_waypoint(
        &mut self,
        service: impl EventService,
        drone_id: String,
        mission_id: String,
        wp: &Waypoint,
        waypoint_idx: i32,
    ) -> Result<(), anyhow::Error> {
        println!("execute waypoint: {:?}", wp);
        match wp {
            Waypoint::Goto(GoToWaypoint {
                distance, height, ..
            }) => {
                if let Some(d) = self.drone.borrow_mut() {
                    DroneTwin::emit_drone_started_to_next_waypoint(
                        service.clone(),
                        drone_id.clone(),
                        mission_id.clone(),
                        waypoint_idx,
                    )
                    .await?;

                    let target_height = *height as i32;
                    let z = target_height - d.odometry.z.round() as i32;
                    let distance = (distance * 100.0).round();
                    let mut fife_meters = ((distance / 500.0).floor() as i32).max(0);
                    let mut rest = (distance % 500.0) as i32;
                    let mut do_480_step = false;

                    // if the last movement would be smaller than
                    if rest < 20 && fife_meters > 0 {
                        fife_meters -= 1;
                        do_480_step = true;
                        rest += 20;
                    }

                    // only the last segment will move in the Z direction to avoid < 20 cm movements
                    let rest_z = (!do_480_step && fife_meters == 0)
                        .then(|| z)
                        .unwrap_or_default();
                    d.go_to(rest, 0, rest_z, 100)
                        .await
                        .map_err(anyhow::Error::msg)?;

                    if do_480_step {
                        let d_z = (fife_meters == 0).then(|| z).unwrap_or_default();
                        d.go_to(480, 0, d_z, 100)
                            .await
                            .map_err(anyhow::Error::msg)?;
                    }
                    for i in 0..fife_meters {
                        let d_z = (i == fife_meters - 1).then(|| z).unwrap_or_default();
                        d.go_to(500, 0, d_z, 100)
                            .await
                            .map_err(anyhow::Error::msg)?;
                    }

                    // d.forward(0).await.map_err(anyhow::Error::msg)?;
                } else {
                    return Err(anyhow::Error::msg("no drone connected".to_string()));
                }

                DroneTwin::emit_drone_arrived_at_waypoint(
                    service.clone(),
                    drone_id,
                    mission_id,
                    waypoint_idx,
                )
                .await?;
                Ok(())
            }
            Waypoint::Turn(TurnWaypoint { deg, .. }) => {
                if let Some(d) = self.drone.borrow_mut() {
                    DroneTwin::emit_drone_started_to_next_waypoint(
                        service.clone(),
                        drone_id.clone(),
                        mission_id.clone(),
                        waypoint_idx,
                    )
                    .await?;

                    let deg = *deg;
                    if deg > 0 {
                        d.cw(deg as u32).await.map_err(anyhow::Error::msg)?
                    } else {
                        d.ccw((-deg) as u32).await.map_err(anyhow::Error::msg)?
                    }
                } else {
                    return Err(anyhow::Error::msg("no drone connected".to_string()));
                }

                DroneTwin::emit_drone_arrived_at_waypoint(
                    service.clone(),
                    drone_id,
                    mission_id,
                    waypoint_idx,
                )
                .await?;
                Ok(())
            }
            Waypoint::Delay(DelayWaypoint { duration, .. }) => {
                DroneTwin::emit_drone_started_to_next_waypoint(
                    service.clone(),
                    drone_id.clone(),
                    mission_id.clone(),
                    waypoint_idx,
                )
                .await?;

                sleep(Duration::from_millis(*duration as u64)).await;

                DroneTwin::emit_drone_arrived_at_waypoint(
                    service.clone(),
                    drone_id,
                    mission_id,
                    waypoint_idx,
                )
                .await?;
                Ok(())
            }
        }
    }

    pub async fn land(&mut self) -> Result<(), String> {
        if let Some(drone) = self.drone.as_ref() {
            drone.land().await?;
            Ok(())
        } else {
            Err("can't land !?".to_string())
        }
    }

    pub fn pos(&self) -> Odometry {
        self.drone
            .as_ref()
            .map(|d| d.odometry.clone())
            .unwrap_or_default()
    }
}
