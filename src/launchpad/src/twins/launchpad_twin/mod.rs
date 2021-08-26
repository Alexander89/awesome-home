use self::events as ev;
use crate::twin::Twin;
use crate::twin::{mk_publish_request, tag_with_id};
use actyx_sdk::service::{EventService, PublishResponse};
use actyx_sdk::{Event, Payload, TagSet};

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
        if let Ok(ev) = event.extract::<ev::LaunchPadEvent>() {
            match ev.payload {
                ev::LaunchPadEvent::DroneMounted(e) => Self::State {
                    id: e.id,
                    ready_to_launch: true,
                    mission: state.mission,
                    mounted_drone: Some(e.drone),
                    drone_enabled: false,
                },
                ev::LaunchPadEvent::LaunchPadRegistered(e) => Self::State {
                    id: e.id,
                    ready_to_launch: state.ready_to_launch,
                    mission: state.mission,
                    mounted_drone: state.mounted_drone,
                    drone_enabled: state.drone_enabled,
                },
                ev::LaunchPadEvent::DroneActivated(e) => Self::State {
                    id: e.id,
                    ready_to_launch: state.ready_to_launch,
                    mission: state.mission,
                    mounted_drone: Some(e.drone),
                    drone_enabled: true,
                },
                ev::LaunchPadEvent::ActivateDroneTimeout(e) => Self::State {
                    id: e.id,
                    ready_to_launch: state.ready_to_launch,
                    mission: state.mission,
                    mounted_drone: state.mounted_drone,
                    drone_enabled: false,
                },
                ev::LaunchPadEvent::DroneStarted(e) => Self::State {
                    id: e.id,
                    ready_to_launch: false,
                    mission: state.mission,
                    mounted_drone: None,
                    drone_enabled: false,
                },
                ev::LaunchPadEvent::MissionQueued(e) => Self::State {
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

pub fn tag_launchpad_id<T>(id: &T) -> TagSet
where
    T: core::fmt::Display,
{
    tag_with_id("launchpad", &id)
}

impl LaunchpadTwin {
    #[allow(dead_code)]
    pub async fn emit_launchpad_registered(
        service: impl EventService,
        id: String,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_launchpad_id(&id),
                &ev::LaunchPadEvent::LaunchPadRegistered(ev::LaunchPadRegisteredEvent { id }),
            ))
            .await
    }

    #[allow(dead_code)]
    pub async fn emit_drone_started(
        service: impl EventService,
        id: String,
        drone: String,
        mission_id: String,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_launchpad_id(&id),
                &ev::LaunchPadEvent::DroneStarted(ev::DroneStartedEvent {
                    id,
                    drone,
                    mission_id,
                }),
            ))
            .await
    }

    #[allow(dead_code)]
    pub async fn emit_activate_drone_timeout(
        service: impl EventService,
        id: String,
        drone: String,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_launchpad_id(&id),
                &ev::LaunchPadEvent::ActivateDroneTimeout(ev::ActivateDroneTimeoutEvent {
                    id,
                    drone,
                }),
            ))
            .await
    }

    #[allow(dead_code)]
    pub async fn emit_drone_activated(
        service: impl EventService,
        id: String,
        drone: String,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_launchpad_id(&id),
                &ev::LaunchPadEvent::DroneActivated(ev::DroneActivatedEvent { id, drone }),
            ))
            .await
    }
}
