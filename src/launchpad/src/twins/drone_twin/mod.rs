use std::convert::TryInto;

use self::ev::Position;
use self::events as ev;
use self::states::UsedState;
use crate::twin::Twin;
use crate::twin::{mk_publish_request, tag_with_id};
use actyx_sdk::service::{EventService, PublishResponse};
use actyx_sdk::{tag, Event, Payload, TagSet};

pub mod events;
pub mod states;

#[derive(Clone)]
pub struct DroneTwin {
    pub id: String,
}

impl Twin for DroneTwin {
    type State = states::DroneTwinState;
    fn name(&self) -> String {
        "launchpad".to_string()
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn query(&self) -> actyx_sdk::language::Query {
        format!("FROM 'launchpad:{}'", self.id)
            .parse()
            .expect("DroneTwin: AQL query not parse-able")
    }

    fn reducer(state: Self::State, event: Event<Payload>) -> Self::State {
        //println!("{:?}", event.payload.json_value());
        if let Ok(ev) = event.extract::<ev::DroneEvent>() {
            match ev.payload {
                ev::DroneEvent::DroneDefined(e) => {
                    states::DroneTwinState::Ready(states::ReadyState {
                        id: e.id,
                        ip: e.ip,
                        ssid: e.ssid,
                        battery: 100,
                        connected: false,
                    })
                }
                ev::DroneEvent::DroneReady(e) => DroneTwin::handle_ready_event(state, e),
                ev::DroneEvent::DroneConnected(e) => DroneTwin::handle_connected_event(state, e),
                ev::DroneEvent::DroneStatsUpdated(e) => {
                    DroneTwin::handle_states_updated_event(state, e)
                }
                ev::DroneEvent::DroneLaunched(e) => DroneTwin::handle_launched_event(state, e),
                ev::DroneEvent::DroneStartedToNextWaypoint(e) => {
                    DroneTwin::handle_started_to_next_waypoint(state, e)
                }
                ev::DroneEvent::DroneArrivedAtWaypoint(e) => {
                    DroneTwin::handle_arrived_at_waypoint(state, e)
                }
                ev::DroneEvent::DroneMissionCompleted(e) => {
                    DroneTwin::handle_mission_completed(state, e)
                }
                ev::DroneEvent::DroneLanded(e) => DroneTwin::handle_landed(state, e),
                ev::DroneEvent::DroneDisconnected(_) => state,
            }
        } else {
            state
        }
    }
}

impl DroneTwin {
    fn handle_ready_event(
        state: states::DroneTwinState,
        _: events::DroneReadyEvent,
    ) -> states::DroneTwinState {
        match state {
            states::DroneTwinState::Undefined(_) => state,
            states::DroneTwinState::Ready(_) => state,
            states::DroneTwinState::Launched(_) => state,
            states::DroneTwinState::Used(state) => {
                states::DroneTwinState::Ready(states::ReadyState {
                    id: state.id,
                    ip: state.ip,
                    ssid: state.ssid,
                    battery: state.battery,
                    connected: false,
                })
            }
        }
    }
    fn handle_connected_event(
        state: states::DroneTwinState,
        _: events::DroneConnectedEvent,
    ) -> states::DroneTwinState {
        match state {
            states::DroneTwinState::Undefined(_) => state,
            states::DroneTwinState::Ready(mut state) => {
                state.connected = true;
                return states::DroneTwinState::Ready(state);
            }
            states::DroneTwinState::Launched(_) => state,
            states::DroneTwinState::Used(state) => {
                states::DroneTwinState::Ready(states::ReadyState {
                    id: state.id,
                    ip: state.ip,
                    ssid: state.ssid,
                    battery: state.battery,
                    connected: true,
                })
            }
        }
    }
    fn handle_states_updated_event(
        state: states::DroneTwinState,
        e: events::DroneStatsUpdatedEvent,
    ) -> states::DroneTwinState {
        match state {
            states::DroneTwinState::Undefined(_) => state,
            states::DroneTwinState::Ready(mut state) => {
                state.battery = e.battery;
                states::DroneTwinState::Ready(state)
            }
            states::DroneTwinState::Launched(mut state) => {
                state.battery = e.battery;
                states::DroneTwinState::Launched(state)
            }
            states::DroneTwinState::Used(mut state) => {
                state.battery = e.battery;
                states::DroneTwinState::Used(state)
            }
        }
    }
    fn handle_launched_event(
        state: states::DroneTwinState,
        e: events::DroneLaunchedEvent,
    ) -> states::DroneTwinState {
        match state {
            states::DroneTwinState::Undefined(_) => state,
            states::DroneTwinState::Ready(s) => {
                states::DroneTwinState::Launched(states::LaunchedState {
                    id: e.id,
                    ip: s.ip,
                    ssid: s.ssid,
                    battery: s.battery,
                    at_waypoint_id: 0,
                    mission_id: e.mission_id,
                    target_waypoint_id: None,
                    completed: false,
                })
            }
            states::DroneTwinState::Launched(_) => state,
            states::DroneTwinState::Used(s) => {
                states::DroneTwinState::Launched(states::LaunchedState {
                    id: e.id,
                    ip: s.ip,
                    ssid: s.ssid,
                    battery: s.battery,
                    at_waypoint_id: 0,
                    mission_id: e.mission_id,
                    target_waypoint_id: None,
                    completed: false,
                })
            }
        }
    }
    fn handle_started_to_next_waypoint(
        state: states::DroneTwinState,
        e: events::DroneStartedToNextWaypointEvent,
    ) -> states::DroneTwinState {
        match state {
            states::DroneTwinState::Undefined(_) => state,
            states::DroneTwinState::Ready(s) => {
                states::DroneTwinState::Launched(states::LaunchedState {
                    id: e.id,
                    ip: s.ip,
                    ssid: s.ssid,
                    battery: s.battery,
                    at_waypoint_id: (e.waypoint_id.max(1) - 1).try_into().unwrap(),
                    mission_id: e.mission_id,
                    target_waypoint_id: Some(e.waypoint_id.try_into().unwrap()),
                    completed: false,
                })
            }
            states::DroneTwinState::Launched(s) => {
                states::DroneTwinState::Launched(states::LaunchedState {
                    id: e.id,
                    ip: s.ip,
                    ssid: s.ssid,
                    battery: s.battery,
                    at_waypoint_id: s.target_waypoint_id.unwrap_or(0),
                    mission_id: e.mission_id,
                    target_waypoint_id: Some(e.waypoint_id.try_into().unwrap()),
                    completed: false,
                })
            }
            states::DroneTwinState::Used(s) => {
                states::DroneTwinState::Launched(states::LaunchedState {
                    id: e.id,
                    ip: s.ip,
                    ssid: s.ssid,
                    battery: s.battery,
                    at_waypoint_id: (e.waypoint_id.max(1) - 1).try_into().unwrap(),
                    mission_id: e.mission_id,
                    target_waypoint_id: Some(e.waypoint_id.try_into().unwrap()),
                    completed: false,
                })
            }
        }
    }
    fn handle_arrived_at_waypoint(
        state: states::DroneTwinState,
        e: events::DroneArrivedAtWaypointEvent,
    ) -> states::DroneTwinState {
        match state {
            states::DroneTwinState::Undefined(_) => state,
            states::DroneTwinState::Ready(s) => {
                states::DroneTwinState::Launched(states::LaunchedState {
                    id: e.id,
                    ip: s.ip,
                    ssid: s.ssid,
                    battery: s.battery,
                    at_waypoint_id: e.waypoint_id as u32,
                    mission_id: e.mission_id,
                    target_waypoint_id: None,
                    completed: false,
                })
            }
            states::DroneTwinState::Launched(mut s) => {
                s.at_waypoint_id = e.waypoint_id as u32;
                s.mission_id = e.mission_id;
                s.target_waypoint_id = None;

                states::DroneTwinState::Launched(s)
            }
            states::DroneTwinState::Used(s) => {
                states::DroneTwinState::Launched(states::LaunchedState {
                    id: e.id,
                    ip: s.ip,
                    ssid: s.ssid,
                    battery: s.battery,
                    at_waypoint_id: e.waypoint_id as u32,
                    mission_id: e.mission_id,
                    target_waypoint_id: None,
                    completed: false,
                })
            }
        }
    }
    fn handle_mission_completed(
        state: states::DroneTwinState,
        _: events::DroneMissionCompletedEvent,
    ) -> states::DroneTwinState {
        match state {
            states::DroneTwinState::Undefined(_) => state,
            states::DroneTwinState::Ready(_) => state,
            states::DroneTwinState::Launched(mut s) => {
                s.completed = false;
                s.target_waypoint_id = None;
                states::DroneTwinState::Launched(s)
            }
            states::DroneTwinState::Used(_) => state,
        }
    }
    fn handle_landed(
        state: states::DroneTwinState,
        _: events::DroneLandedEvent,
    ) -> states::DroneTwinState {
        match state {
            states::DroneTwinState::Undefined(_) => state,
            states::DroneTwinState::Ready(_) => state,
            states::DroneTwinState::Launched(mut s) => {
                s.completed = false;
                s.target_waypoint_id = None;
                states::DroneTwinState::Used(UsedState {
                    id: s.id,
                    ip: s.ip,
                    ssid: s.ssid,
                    last_mission_id: s.mission_id,
                    battery: s.battery,
                })
            }
            states::DroneTwinState::Used(_) => state,
        }
    }
}

pub fn tag_drone_id<T>(id: &T) -> TagSet
where
    T: core::fmt::Display,
{
    tag_with_id("drone", &id)
}

pub fn tag_drone_mission_started<T>(id: &T) -> TagSet
where
    T: core::fmt::Display,
{
    tag_drone_id(id) + tag!("drone.mission.started")
}

pub fn tag_drone_mission_completed<T>(id: &T) -> TagSet
where
    T: core::fmt::Display,
{
    tag_drone_id(id) + tag!("drone.mission.completed")
}

impl DroneTwin {
    #[allow(dead_code)]
    pub async fn emit_drone_ready(
        service: impl EventService,
        id: String,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_drone_id(&id),
                &ev::DroneEvent::DroneReady(ev::DroneReadyEvent { id }),
            ))
            .await
    }
    #[allow(dead_code)]
    pub async fn emit_drone_connected(
        service: impl EventService,
        id: String,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_drone_id(&id),
                &ev::DroneEvent::DroneConnected(ev::DroneConnectedEvent { id }),
            ))
            .await
    }

    #[allow(dead_code)]
    pub async fn emit_drone_stats_updated(
        service: impl EventService,
        id: String,
        battery: u8,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_drone_id(&id),
                &ev::DroneEvent::DroneStatsUpdated(ev::DroneStatsUpdatedEvent { id, battery }),
            ))
            .await
    }
    #[allow(dead_code)]
    pub async fn emit_drone_launched(
        service: impl EventService,
        id: String,
        mission_id: String,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_drone_mission_started(&id),
                &ev::DroneEvent::DroneLaunched(ev::DroneLaunchedEvent { id, mission_id }),
            ))
            .await
    }
    #[allow(dead_code)]
    pub async fn emit_drone_started_to_next_waypoint(
        service: impl EventService,
        id: String,
        mission_id: String,
        waypoint_id: i32,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_drone_id(&id),
                &ev::DroneEvent::DroneStartedToNextWaypoint(ev::DroneStartedToNextWaypointEvent {
                    id,
                    mission_id,
                    waypoint_id,
                }),
            ))
            .await
    }
    #[allow(dead_code)]
    pub async fn emit_drone_arrived_at_waypoint(
        service: impl EventService,
        id: String,
        mission_id: String,
        waypoint_id: i32,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_drone_id(&id),
                &ev::DroneEvent::DroneArrivedAtWaypoint(ev::DroneArrivedAtWaypointEvent {
                    id,
                    mission_id,
                    waypoint_id,
                }),
            ))
            .await
    }
    #[allow(dead_code)]
    pub async fn emit_drone_mission_completed(
        service: impl EventService,
        id: String,
        mission_id: String,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_drone_mission_completed(&id),
                &ev::DroneEvent::DroneMissionCompleted(ev::DroneMissionCompletedEvent {
                    id,
                    mission_id,
                }),
            ))
            .await
    }
    #[allow(dead_code)]
    pub async fn emit_drone_landed(
        service: impl EventService,
        id: String,
        x: f32,
        y: f32,
        z: f32,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_drone_id(&id),
                &ev::DroneEvent::DroneLanded(ev::DroneLandedEvent {
                    id,
                    at: Position { x, y, z },
                }),
            ))
            .await
    }
    #[allow(dead_code)]
    pub async fn emit_drone_disconnected(
        service: impl EventService,
        id: String,
    ) -> Result<PublishResponse, anyhow::Error> {
        service
            .publish(mk_publish_request(
                tag_drone_id(&id),
                &ev::DroneEvent::DroneDisconnected(ev::DroneDisconnectedEvent { id }),
            ))
            .await
    }
}
