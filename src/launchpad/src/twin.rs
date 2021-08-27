#![allow(dead_code)]
use actyx_sdk::{
    language::Query,
    service::{
        EventService, Order, PublishEvent, PublishRequest, QueryRequest, QueryResponse, SessionId,
        StartFrom, SubscribeMonotonicRequest, SubscribeMonotonicResponse,
    },
    Event, EventKey, Metadata, OffsetMap, Payload, Tag, TagSet,
};
use futures::StreamExt;
use std::{
    fmt::Debug,
    str::FromStr,
    thread::sleep,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tokio_stream_ext::{combine_latest, switch_map, StreamOpsExt};

pub trait Twin: Clone + Send + Sync {
    type State: Debug + Default + Clone + Send + Sized + Sync + Unpin + PartialEq + 'static;

    fn name(&self) -> String;
    fn id(&self) -> String;
    fn query(&self) -> Query;

    fn reducer(state: Self::State, event: Event<Payload>) -> Self::State;
}

pub fn execute_twin<S, T>(event_service: S, twin: T) -> TwinExecuter<T::State>
where
    S: EventService + Sync + 'static,
    T: Twin + Sync + 'static,
{
    let (tx, rx) = mpsc::channel::<T::State>(100);
    tokio::spawn(async move {
        'subscription: loop {
            let launchpad_subscription = event_service
                .subscribe_monotonic(SubscribeMonotonicRequest {
                    session: SessionId::from(format!("{}:{}", twin.name(), twin.id())),
                    from: StartFrom::LowerBound(OffsetMap::empty()),
                    query: twin.query(),
                })
                .await;

            let mut state: T::State = Default::default();
            let _ = tx.send(state.clone()).await;
            if let Ok(mut stream) = launchpad_subscription {
                'eventLoop: loop {
                    match stream.next().await {
                        Some(SubscribeMonotonicResponse::Event { event, .. }) => {
                            let key = EventKey {
                                lamport: event.lamport,
                                stream: event.stream,
                                offset: event.offset,
                            };
                            let meta = Metadata {
                                timestamp: event.timestamp,
                                tags: event.tags,
                                app_id: event.app_id,
                            };
                            let payload = event.payload;
                            let event = Event::<Payload> { key, meta, payload };
                            state = T::reducer(state, event);
                            let _ = tx.send(state.clone()).await;
                        }
                        Some(SubscribeMonotonicResponse::Offsets(..)) => {
                            let _ = tx.send(state.clone()).await;
                        }
                        Some(SubscribeMonotonicResponse::TimeTravel { .. }) => {
                            break 'eventLoop;
                        }
                        None => {
                            println!("stream terminated?");
                            sleep(Duration::from_millis(100));
                            drop(tx);
                            break 'subscription;
                        }
                        Some(other) => {
                            println!("event {:?} ", other);
                        }
                    }
                }
            }
        }
    });
    TwinExecuter::new(rx, 100)
}

pub async fn current_state<S, T>(event_service: S, twin: T) -> Box<Result<T::State, anyhow::Error>>
where
    T: Twin + Clone + Sync + 'static,
    S: EventService + Send + Sync + 'static,
{
    let event_stream = event_service
        .query(QueryRequest {
            lower_bound: None,
            upper_bound: None,
            query: twin.query(),
            order: Order::Asc,
        })
        .await;

    let mut state: T::State = Default::default();

    match event_stream {
        Ok(mut stream) => 'eventLoop: loop {
            match stream.next().await {
                Some(QueryResponse::Event(event)) => {
                    let key = EventKey {
                        lamport: event.lamport,
                        stream: event.stream,
                        offset: event.offset,
                    };
                    let meta = Metadata {
                        timestamp: event.timestamp,
                        tags: event.tags,
                        app_id: event.app_id,
                    };
                    let payload = event.payload;
                    let event = Event::<Payload> { key, meta, payload };
                    state = T::reducer(state, event);
                }
                None => {
                    break 'eventLoop Box::new(Ok(state.clone()));
                }
                _ => {}
            }
        },
        Err(e) => Box::new(Err(e)),
    }
}

#[derive(Debug)]
pub struct TwinExecuter<S>
where
    S: Clone + Send + Sync,
{
    last_interaction: Instant,
    input: mpsc::Receiver<S>,
    last_state: Option<S>,
    debounce_time_ms: u64,
}

impl<S> TwinExecuter<S>
where
    S: Debug + Clone + Send + Sync + Unpin + PartialEq,
{
    pub fn new(input: mpsc::Receiver<S>, debounce_time_ms: u64) -> Self {
        Self {
            last_interaction: Instant::now(),
            input,
            debounce_time_ms,
            last_state: None,
        }
    }

    pub fn as_stream(self) -> impl Stream<Item = S> + Unpin {
        Box::pin(
            ReceiverStream::new(self.input)
                .debounce(Duration::from_millis(80))
                .distinct_until_changed(),
        )
    }
}

pub fn spawn_observer<T, F>(mut stream: T, state_changed: F) -> Observation
where
    F: Fn(T::Item) -> () + Send + 'static,
    T: Stream + Unpin + Send + 'static,
{
    let (command_sender, mut tx) = tokio::sync::mpsc::channel(1);
    let handler = tokio::spawn(async move {
        'observeLaunchpad: loop {
            tokio::select! {
                res = stream.next() => {
                    match res {
                        Some(state) => state_changed(state),
                        None => break 'observeLaunchpad,
                    }
                }
                _ = tx.recv() => {
                    break 'observeLaunchpad;
                }
            };
        }
        println!("I'm done here");
    });

    Observation {
        command_sender,
        handler,
    }
}

pub fn resolve_registry<Actyx, Registry, Entity>(
    service: Actyx,
    registry_twin: Registry,
    map_to_entity: fn(Registry::State) -> Vec<Entity>,
) -> impl Stream<Item = Vec<Entity::State>>
where
    Actyx: EventService + Clone + Sync + 'static,
    Registry: Twin + 'static,
    Entity: Twin + 'static,
{
    switch_map(
        execute_twin(service.clone(), registry_twin).as_stream(),
        move |state| {
            let l = combine_latest(
                (map_to_entity)(state)
                    .iter()
                    .map(|entity| execute_twin(service.clone(), entity.clone()).as_stream())
                    .collect(),
            );
            Some(l)
        },
    )
}

pub fn resolve_relation<A, T, E>(
    service: A,
    registry_twin: T,
    map_to_entity: fn(T::State) -> Option<E>,
) -> impl Stream<Item = E::State>
where
    A: EventService + Clone + Sync + 'static,
    T: Twin + 'static, // Stream<Item = OS> + Unpin + Send + 'static,
    E: Twin + 'static,
{
    switch_map(
        execute_twin(service.clone(), registry_twin).as_stream(),
        move |state| {
            let twin = (map_to_entity)(state);
            twin.map(|e| execute_twin(service.clone(), e).as_stream())
        },
    )
}
pub struct Observation {
    command_sender: mpsc::Sender<bool>,
    handler: tokio::task::JoinHandle<()>,
}

impl Into<tokio::task::JoinHandle<()>> for Observation {
    fn into(self) -> tokio::task::JoinHandle<()> {
        self.handler
    }
}

impl Observation {
    pub async fn cancel(&self) -> Result<(), mpsc::error::SendError<bool>> {
        self.command_sender.send(true).await
    }

    pub async fn cancel_blocking(self) -> bool {
        let (a, b) = tokio::join!(self.command_sender.send(true), self.handler);
        a.is_ok() && b.is_ok()
    }

    pub fn handler(&self) -> &tokio::task::JoinHandle<()> {
        &self.handler
    }

    pub fn as_handler(self) -> tokio::task::JoinHandle<()> {
        self.handler
    }

    pub fn extract(self) -> (tokio::task::JoinHandle<()>, mpsc::Sender<bool>) {
        (self.handler, self.command_sender)
    }
}

pub fn tag_with_id<T>(base: &str, id: &T) -> TagSet
where
    T: core::fmt::Display,
{
    let base_tag = Tag::from_str(base).unwrap();
    let id_tag = base_tag.clone() + format!(":{}", id);
    TagSet::from(vec![base_tag, id_tag])
}

pub fn mk_publish_request<T>(tags: TagSet, event: &T) -> PublishRequest
where
    T: serde::Serialize,
{
    PublishRequest {
        data: vec![PublishEvent {
            tags,
            payload: Payload::compact(&serde_json::json!(event)).unwrap(),
        }],
    }
}
