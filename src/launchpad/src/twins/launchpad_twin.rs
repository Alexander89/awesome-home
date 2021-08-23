use super::twin::Twin;
use actyx_sdk::{service::EventResponse, Payload};

#[derive(Clone, Debug)]
pub struct LaunchpadTwinState {
    pub id: String,
    pub ready_to_launch: bool,
}

impl Default for LaunchpadTwinState {
    fn default() -> Self {
        Self {
            id: Default::default(),
            ready_to_launch: Default::default(),
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
        format!("FROM allEvents | 'launchpad:{}'", self.id)
            .parse()
            .expect("LaunchpadTwin: AQL query not parse-able")
    }

    fn reducer(state: Self::State, event: EventResponse<Payload>) -> Self::State {
        //println!("{:?}", event.payload.json_value());
        state
    }
}
