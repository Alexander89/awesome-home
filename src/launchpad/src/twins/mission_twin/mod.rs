use super::twin::Twin;
use crate::twins::mission_twin::events::MissionEvent;
use crate::twins::mission_twin::types::Waypoint;
use actyx_sdk::{Event, Payload};
use std::collections::HashSet;
pub mod events;
pub mod types;

#[derive(Clone, Debug)]
pub struct MissionTwinState {
    pub id: String,
    pub name: String,
    pub waypoints: Vec<Waypoint>,
    pub visible: bool,
}

impl Default for MissionTwinState {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            waypoints: vec![],
            visible: true,
        }
    }
}

#[derive(Clone)]
pub struct MissionTwin {
    pub id: String,
}

impl Twin for MissionTwin {
    type State = MissionTwinState;
    fn name(&self) -> String {
        "mission".to_string()
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn query(&self) -> actyx_sdk::language::Query {
        format!("FROM 'mission:{}'", self.id)
            .parse()
            .expect("MissionTwin: AQL query not parse-able")
    }
    fn reducer(state: Self::State, event: Event<Payload>) -> Self::State {
        //println!("{:?}", event.payload.json_value());
        if let Ok(ev) = event.extract::<MissionEvent>() {
            match ev.payload {
                MissionEvent::DefineMission(e) => Self::State {
                    id: e.id,
                    name: e.name,
                    waypoints: e.waypoints,
                    visible: state.visible,
                },
                MissionEvent::ShowMission(e) => Self::State {
                    id: state.id,
                    name: state.name,
                    waypoints: state.waypoints,
                    visible: e.visible,
                },
            }
        } else {
            state
        }
    }
}

#[derive(Clone)]
pub struct MissionRegistryTwin {}

impl Twin for MissionRegistryTwin {
    type State = HashSet<String>;
    fn name(&self) -> String {
        "missionRegistry".to_string()
    }
    fn id(&self) -> String {
        "reg".to_string()
    }
    fn query(&self) -> actyx_sdk::language::Query {
        format!("FROM 'mission'")
            .parse()
            .expect("MissionTwin: AQL query not parse-able")
    }
    fn reducer(mut state: Self::State, event: Event<Payload>) -> Self::State {
        //println!("{:?}", event.payload.json_value());
        if let Ok(ev) = event.extract::<MissionEvent>() {
            match ev.payload {
                MissionEvent::DefineMission(e) => {
                    state.insert(e.id);
                    state
                }
                MissionEvent::ShowMission(e) => {
                    state.remove(&e.id);
                    state
                }
            }
        } else {
            state
        }
    }
}
