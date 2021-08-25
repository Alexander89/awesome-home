use crate::twins::mission_twin::types::Waypoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefineMissionEvent {
    pub id: String,
    pub name: String,
    pub waypoints: Vec<Waypoint>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowMissionEvent {
    pub id: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "eventType")]
#[serde(rename_all = "camelCase")]
pub enum MissionEvent {
    DefineMission(DefineMissionEvent),
    ShowMission(ShowMissionEvent),
}
