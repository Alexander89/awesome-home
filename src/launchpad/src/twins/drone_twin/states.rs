#[derive(Clone, Debug, PartialEq)]
pub struct UndefinedState {
    pub id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReadyState {
    pub id: String,
    pub ip: String,
    pub ssid: String,
    pub battery: u8,
    pub connected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaunchedState {
    pub id: String,
    pub ip: String,
    pub ssid: String,
    pub mission_id: String,
    pub at_waypoint_id: u32,
    pub target_waypoint_id: Option<u32>,
    pub completed: bool,
    pub battery: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UsedState {
    pub id: String,
    pub ip: String,
    pub ssid: String,
    pub last_mission_id: String,
    pub battery: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DroneTwinState {
    Undefined(UndefinedState),
    Ready(ReadyState),
    Launched(LaunchedState),
    Used(UsedState),
}

impl Default for DroneTwinState {
    fn default() -> Self {
        DroneTwinState::Undefined(UndefinedState {
            id: Default::default(),
        })
    }
}
