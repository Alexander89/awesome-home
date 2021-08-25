use crate::twin::Twin;
use crate::twins::launchpad_twin::events::LaunchPadEvent;
use actyx_sdk::{Event, Payload};
pub mod events;

#[derive(Clone, Debug)]
pub struct LaunchpadTwinState {
    pub id: String,
    pub ready_to_launch: bool,
    pub mission: Option<String>,
    pub mounted_drone: Option<String>,
    pub drone_enabled: bool,
}

impl Default for LaunchpadTwinState {
    fn default() -> Self {
        Self {
            id: Default::default(),
            ready_to_launch: Default::default(),
            mission: None,
            mounted_drone: None,
            drone_enabled: false,
        }
    }
}

#[derive(Clone)]
pub struct LaunchpadTwin {
    pub id: String,
}

impl Twin for LaunchpadTwin {
    type State = LaunchpadTwinState;
    fn name(&self) -> String {
        "launchpad".to_string()
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn query(&self) -> actyx_sdk::language::Query {
        format!("FROM 'launchpad:{}'", self.id)
            .parse()
            .expect("LaunchpadTwin: AQL query not parse-able")
    }

    fn reducer(state: Self::State, event: Event<Payload>) -> Self::State {
        //println!("{:?}", event.payload.json_value());
        if let Ok(ev) = event.extract::<LaunchPadEvent>() {
            match ev.payload {
                LaunchPadEvent::DroneMounted(e) => Self::State {
                    id: e.id,
                    ready_to_launch: true,
                    mission: state.mission,
                    mounted_drone: Some(e.drone),
                    drone_enabled: false,
                },
                LaunchPadEvent::LaunchPadRegistered(e) => Self::State {
                    id: e.id,
                    ready_to_launch: state.ready_to_launch,
                    mission: state.mission,
                    mounted_drone: state.mounted_drone,
                    drone_enabled: state.drone_enabled,
                },
                LaunchPadEvent::DroneActivated(e) => Self::State {
                    id: e.id,
                    ready_to_launch: state.ready_to_launch,
                    mission: state.mission,
                    mounted_drone: Some(e.drone),
                    drone_enabled: true,
                },
                LaunchPadEvent::ActivateDroneTimeout(e) => Self::State {
                    id: e.id,
                    ready_to_launch: state.ready_to_launch,
                    mission: state.mission,
                    mounted_drone: state.mounted_drone,
                    drone_enabled: false,
                },
                LaunchPadEvent::DroneStarted(e) => Self::State {
                    id: e.id,
                    ready_to_launch: false,
                    mission: state.mission,
                    mounted_drone: None,
                    drone_enabled: false,
                },
                LaunchPadEvent::MissionQueued(e) => Self::State {
                    id: e.launchpad_id,
                    ready_to_launch: state.ready_to_launch,
                    mission: Some(e.mission_id),
                    mounted_drone: state.mounted_drone,
                    drone_enabled: state.drone_enabled,
                },
            }
        } else {
            state
        }
    }
}
