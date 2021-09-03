use self::events as ev;
use crate::twin::Twin;
use crate::twin::{mk_publish_request, tag_with_id};
use actyx_sdk::service::{EventService, PublishResponse};
use actyx_sdk::{Event, Payload, TagSet};
pub mod events;

#[derive(Clone, Debug, PartialEq)]
pub struct LaunchpadTwinState {
    pub id: String,
    pub current_mission: Option<String>,
    pub mission_queue: Vec<String>,
    pub attached_drone: Option<String>,
}

impl Default for LaunchpadTwinState {
    fn default() -> Self {
        Self {
            id: Default::default(),
            current_mission: None,
            mission_queue: Vec::new(),
            attached_drone: None,
        }
    }
}

#[derive(Clone)]
pub struct LaunchpadTwin {
    pub id: String,
}

impl LaunchpadTwin {
    pub fn new(id: String) -> Self {
        Self { id }
    }
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
        format!("FROM 'launchpad:{}' | 'drone.mission.completed'", self.id)
            .parse()
            .expect("LaunchpadTwin: AQL query not parse-able")
    }

    fn reducer(state: Self::State, event: Event<Payload>) -> Self::State {
        //println!("{:?}", event.payload.json_value());
        if let Ok(ev) = event.extract::<ev::LaunchPadEvent>() {
            match ev.payload {
                ev::LaunchPadEvent::DroneMounted(e) => Self::State {
                    id: e.id,
                    current_mission: state.current_mission,
                    mission_queue: state.mission_queue,
                    attached_drone: Some(e.drone),
                },
                ev::LaunchPadEvent::LaunchPadRegistered(e) => Self::State {
                    id: e.id,
                    current_mission: state.current_mission,
                    mission_queue: state.mission_queue,
                    attached_drone: state.attached_drone,
                },
                ev::LaunchPadEvent::MissionActivated(e) => Self::State {
                    id: state.id,
                    current_mission: Some(e.mission_id),
                    mission_queue: state.mission_queue,
                    attached_drone: state.attached_drone,
                },
                ev::LaunchPadEvent::DroneMissionCompleted(e) => {
                    if Some(e.id) == state.attached_drone {
                        let mission_queue = state
                            .mission_queue
                            .iter()
                            .filter({
                                let id = e.mission_id.clone();
                                move |m_id| **m_id != id
                            })
                            .map(|s| s.to_owned())
                            .collect::<Vec<String>>();

                        Self::State {
                            id: state.id,
                            current_mission: None,
                            mission_queue,
                            attached_drone: None,
                        }
                    } else {
                        state
                    }
                }
                ev::LaunchPadEvent::MissionQueued(e) => {
                    let mut mission_queue = state.mission_queue.clone();
                    mission_queue.push(e.mission_id);

                    Self::State {
                        id: e.launchpad_id,
                        current_mission: state.current_mission,
                        mission_queue,
                        attached_drone: state.attached_drone,
                    }
                }
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
    pub async fn emit_mission_activated(
        service: impl EventService,
        launchpad_id: String,
        mission_id: String,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_launchpad_id(&launchpad_id),
                &ev::LaunchPadEvent::MissionActivated(ev::MissionActivatedEvent {
                    launchpad_id,
                    mission_id,
                }),
            ))
            .await
    }
}
