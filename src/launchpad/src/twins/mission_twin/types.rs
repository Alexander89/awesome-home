use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoToWaypoint {
    pub map_x: f32,
    pub map_y: f32,
    pub height: i16,
    pub angle: i16,
    pub distance: f32,
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnWaypoint {
    pub deg: i16,
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelayWaypoint {
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Waypoint {
    GoTo(GoToWaypoint),
    Turn(TurnWaypoint),
    Delay(DelayWaypoint),
}
