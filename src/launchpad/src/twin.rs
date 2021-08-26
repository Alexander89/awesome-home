use actyx_sdk::{
    language::Query,
    service::{
        EventService, Order, QueryRequest, QueryResponse, SessionId, StartFrom,
        SubscribeMonotonicRequest, SubscribeMonotonicResponse,
    },
    Event, EventKey, Metadata, OffsetMap, Payload,
};
use core::task::Waker;
use futures::task::Context;
use futures::task::Poll;
use futures::StreamExt;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::{fmt::Debug, sync::Arc};
use std::{pin::Pin, thread, time::Instant};
use std::{thread::sleep, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::Stream;

use crate::twins::{combine_latest::combine_latest, switch_map::switch_map};

pub trait Twin: Clone + Send + Sync {
    type State: Default + Clone + Send + Sized + Sync + Unpin + 'static;

    fn name(&self) -> String;
    fn id(&self) -> String;
    fn query(&self) -> Query;

    fn reducer(state: Self::State, event: Event<Payload>) -> Self::State;
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub async fn current_state<S, T>(event_service: S, twin: T) -> Box<Result<T::State, anyhow::Error>>
where
    T: Twin + Clone + std::marker::Sync + 'static,
    S: EventService + Send + std::marker::Sync + 'static,
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
    S: Clone + Send + std::marker::Sync,
{
    last_interaction: Instant,
    input: mpsc::Receiver<S>,
    last_state: Option<S>,
    debounce_time_ms: u64,

    waker_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    next_trigger: Option<Arc<Mutex<Instant>>>,
    waker: Option<Arc<Mutex<Waker>>>,
}

impl<S> TwinExecuter<S>
where
    S: Clone + Send + std::marker::Sync,
{
    pub fn new(input: mpsc::Receiver<S>, debounce_time_ms: u64) -> Self {
        Self {
            last_interaction: Instant::now(),
            input,
            debounce_time_ms,
            last_state: None,

            waker_thread: Arc::new(Mutex::new(None)),
            next_trigger: None,
            waker: None,
        }
    }
}

impl<S> Stream for TwinExecuter<S>
where
    S: Unpin + Clone + Send + std::marker::Sync,
{
    type Item = S;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // process input to get the latest state and reset the debounce time
        match self.input.poll_recv(cx) {
            Poll::Ready(Some(state)) => {
                #[allow(unused_unsafe)]
                let this: &mut Self = unsafe { Pin::get_mut(self) };
                this.last_state = Some(state);
                this.last_interaction = Instant::now();
                // immediately request next poll to process all events in the input stream
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            // if input stream terminates, the last state could be published immediately
            Poll::Ready(None) => return Poll::Ready(self.last_state.clone()),
            // continue with debounce timer
            Poll::Pending => (),
        };

        if self.last_interaction.elapsed().as_millis() as u64 >= self.debounce_time_ms {
            if let Some(s) = self.last_state.clone() {
                self.last_state = None;
                return Poll::Ready(Some(s));
            }
        } else {
            // assign new waker or replace existing waker
            // Save to switch the Stream to a different thread
            if let Some(waker) = &self.waker {
                let mut waker = waker.lock().unwrap();
                if !waker.will_wake(cx.waker()) {
                    *waker = cx.waker().clone();
                }
            } else {
                let waker = Arc::new(Mutex::new(cx.waker().clone()));
                self.waker = Some(waker.clone());
            }

            // prepare deadline to trigger
            let trigger_in = Duration::from_millis(
                self.debounce_time_ms - self.last_interaction.elapsed().as_millis() as u64,
            );
            let when = Instant::now() + trigger_in;

            if let Some(next) = &self.next_trigger {
                let mut t = next.lock().unwrap();
                *t = when;
            } else {
                self.next_trigger = Some(Arc::new(Mutex::new(when)));
            }

            if let (Some(waker), Some(trigger)) = (&self.waker, &self.next_trigger) {
                let mut thread = self.waker_thread.lock().unwrap();
                // start thread, if no thread is already running
                if thread.is_none() {
                    let waker_thread = self.waker_thread.clone();
                    let waker = waker.clone();
                    let next_trigger = trigger.clone();

                    // Start waker thread to trigger feature after timeout.
                    *thread = Some(thread::spawn(move || {
                        // println!("start Thread");
                        loop {
                            let now = Instant::now();
                            let when = *(next_trigger.lock().unwrap());
                            if now < when {
                                // println!("sleep for {:?}", when - now);
                                thread::sleep(when - now);
                            } else {
                                // no more looping after timeout
                                break;
                            }
                        }
                        let mut t = waker_thread.lock().unwrap();
                        *t = None;

                        // println!("wake up!");
                        let waker = waker.lock().unwrap();
                        waker.wake_by_ref();
                    }));
                }
            }
        }
        Poll::Pending
    }
}

#[allow(dead_code)]
pub fn observe<T, S, F>(mut stream: T, state_changed: F) -> Observation
where
    F: Fn(S) -> () + Send + 'static,
    S: Send,
    T: Stream<Item = S> + std::marker::Unpin + Send + 'static,
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

pub fn observe_registry<A, T, M, E, F>(
    mut service: A,
    registry_twin: T,
    map_to_entity: M,
    state_changed: F,
) -> Observation
where
    A: EventService + Clone + Sync + 'static,
    T: Twin + 'static, // Stream<Item = OS> + std::marker::Unpin + Send + 'static,
    M: Fn(T::State) -> Vec<E> + Clone + Send + 'static,
    E: Twin + 'static,
    F: Fn(Vec<E::State>) -> () + Send + Sync + 'static,
{
    let service = Arc::new(Mutex::new(service));
    let mapper = Arc::new(Mutex::new(map_to_entity));

    let stream = switch_map(
        execute_twin(service.lock().unwrap().clone(), registry_twin),
        {
            let ser = service.clone();
            let m = mapper.clone();
            move |state| {
                let l = combine_latest(
                    (m.lock().unwrap())(state)
                        .iter()
                        .map(|entity| execute_twin(ser.lock().unwrap().clone(), entity.clone()))
                        .collect(),
                );
                Some(l)
            }
        },
    );
    observe(stream, state_changed)
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
    #[allow(dead_code)]
    pub async fn cancel(&self) -> Result<(), mpsc::error::SendError<bool>> {
        self.command_sender.send(true).await
    }
    #[allow(dead_code)]
    pub async fn cancel_blocking(self) -> bool {
        let (a, b) = tokio::join!(self.command_sender.send(true), self.handler);
        a.is_ok() && b.is_ok()
    }
    #[allow(dead_code)]
    pub fn handler(&self) -> &tokio::task::JoinHandle<()> {
        &self.handler
    }
    #[allow(dead_code)]
    pub fn as_handler(self) -> tokio::task::JoinHandle<()> {
        self.handler
    }
    #[allow(dead_code)]
    pub fn extract(self) -> (tokio::task::JoinHandle<()>, mpsc::Sender<bool>) {
        (self.handler, self.command_sender)
    }
}
