use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneDefinedEvent {
    pub id: String,
    pub ssid: String,
    pub ip: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneReadyEvent {
    pub id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneConnectedEvent {
    pub id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneStatsUpdatedEvent {
    pub id: String,
    pub battery: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneLaunchedEvent {
    pub id: String,
    pub mission_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneStartedToNextWaypointEvent {
    pub id: String,
    pub mission_id: String,
    pub waypoint_id: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneArrivedAtWaypointEvent {
    pub id: String,
    pub mission_id: String,
    pub waypoint_id: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneMissionCompletedEvent {
    pub id: String,
    pub mission_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneLandedEvent {
    pub id: String,
    pub at: Position,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneDisconnectedEvent {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "eventType")]
#[serde(rename_all = "camelCase")]
pub enum DroneEvent {
    DroneDefined(DroneDefinedEvent),
    DroneReady(DroneReadyEvent),
    DroneConnected(DroneConnectedEvent),
    DroneStatsUpdated(DroneStatsUpdatedEvent),
    DroneLaunched(DroneLaunchedEvent),
    DroneStartedToNextWaypoint(DroneStartedToNextWaypointEvent),
    DroneArrivedAtWaypoint(DroneArrivedAtWaypointEvent),
    DroneMissionCompleted(DroneMissionCompletedEvent),
    DroneLanded(DroneLandedEvent),
    DroneDisconnected(DroneDisconnectedEvent),
}
