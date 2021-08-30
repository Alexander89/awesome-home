use std::{borrow::BorrowMut, time::Duration};

use actyx_sdk::service::EventService;
use tello::{odometry::Odometry, CommandMode, Drone};
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

    pub fn is_drone_connected(&self) -> bool {
        self.drone.as_ref().is_some()
    }

    pub async fn connect(&mut self, ip: String) -> Result<(), String> {
        let drone = Drone::new(&*ip).command_mode();
        // this always fails
        let _res = drone.enable().await;
        self.drone = Some(drone);
        Ok(())
    }
    pub async fn take_off(&mut self) -> Result<(), String> {
        if let Some(drone) = self.drone.as_ref() {
            drone.take_off().await?;
            Ok(())
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
    ) -> Result<(), anyhow::Error> {
        match wp {
            Waypoint::Goto(GoToWaypoint {
                id,
                distance,
                height,
                duration,
                ..
            }) => {
                if let Some(d) = self.drone.borrow_mut() {
                    let target_height = *height as i32;
                    let z = target_height - d.odometry.z.round() as i32;
                    d.go_to(0, (*distance).round() as i32, z, 100)
                        .await
                        .map_err(anyhow::Error::msg)?
                } else {
                    return Err(anyhow::Error::msg("no drone connected".to_string()));
                }

                let _ = tokio::join!(
                    DroneTwin::emit_drone_started_to_next_waypoint(
                        service.clone(),
                        drone_id.clone(),
                        mission_id.clone(),
                        *id,
                    ),
                    sleep(Duration::from_secs_f32(duration / 1000.0))
                );

                DroneTwin::emit_drone_arrived_at_waypoint(
                    service.clone(),
                    drone_id,
                    mission_id,
                    *id,
                )
                .await
                .map(|_| ())
            }
            Waypoint::Turn(TurnWaypoint {
                id, deg, duration, ..
            }) => {
                if let Some(d) = self.drone.borrow_mut() {
                    let deg = *deg;
                    if deg > 0 {
                        d.cw(deg as u32).await.map_err(anyhow::Error::msg)?
                    } else {
                        d.ccw((-deg) as u32).await.map_err(anyhow::Error::msg)?
                    }
                } else {
                    return Err(anyhow::Error::msg("no drone connected".to_string()));
                }

                let _ = tokio::join!(
                    DroneTwin::emit_drone_started_to_next_waypoint(
                        service.clone(),
                        drone_id.clone(),
                        mission_id.clone(),
                        *id,
                    ),
                    sleep(Duration::from_secs_f32(duration / 1000.0))
                );

                DroneTwin::emit_drone_arrived_at_waypoint(
                    service.clone(),
                    drone_id,
                    mission_id,
                    *id,
                )
                .await
                .map(|_| ())
            }
            Waypoint::Delay(DelayWaypoint { id, duration, .. }) => {
                DroneTwin::emit_drone_started_to_next_waypoint(
                    service.clone(),
                    drone_id.clone(),
                    mission_id.clone(),
                    *id,
                )
                .await?;

                sleep(Duration::from_secs_f32(duration / 1000.0)).await;

                DroneTwin::emit_drone_arrived_at_waypoint(
                    service.clone(),
                    drone_id,
                    mission_id,
                    *id,
                )
                .await
                .map(|_| ())
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
