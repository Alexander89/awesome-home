use crate::twins::drone_twin::events as drone_events;
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
#[serde(rename_all = "camelCase")]
pub struct MissionQueuedEvent {
    pub launchpad_id: String,
    pub mission_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionActivatedEvent {
    pub launchpad_id: String,
    pub mission_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "eventType")]
#[serde(rename_all = "camelCase")]
pub enum LaunchPadEvent {
    LaunchPadRegistered(LaunchPadRegisteredEvent),
    DroneMounted(DroneMountedEvent),
    MissionQueued(MissionQueuedEvent),
    MissionActivated(MissionActivatedEvent),
    DroneMissionCompleted(drone_events::DroneMissionCompletedEvent),
}
