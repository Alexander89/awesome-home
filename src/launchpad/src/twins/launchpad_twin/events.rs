use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchPadRegisteredEvent {
    pub id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneMountedEvent {
    pub id: String,
    pub drone: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneActivatedEvent {
    pub id: String,
    pub drone: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateDroneTimeoutEvent {
    pub id: String,
    pub drone: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DroneStartedEvent {
    pub id: String,
    pub drone: String,
    pub mission_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionQueuedEvent {
    pub mission_id: String,
    pub launchpad_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "eventType")]
#[serde(rename_all = "camelCase")]
pub enum LaunchPadEvent {
    LaunchPadRegistered(LaunchPadRegisteredEvent),
    DroneMounted(DroneMountedEvent),
    DroneActivated(DroneActivatedEvent),
    ActivateDroneTimeout(ActivateDroneTimeoutEvent),
    DroneStarted(DroneStartedEvent),
    MissionQueued(MissionQueuedEvent),
}
