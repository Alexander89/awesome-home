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

pub trait Twin: Clone + Send {
    type State: Default + Clone + Send + Sized + std::marker::Sync + 'static;

    fn name(&self) -> String;
    fn id(&self) -> String;
    fn query(&self) -> Query;

    fn reducer(state: Self::State, event: Event<Payload>) -> Self::State;
}

#[allow(dead_code)]
pub fn execute_twin<S, T>(
    event_service: S,
    twin: T,
) -> Result<TwinExecuter<T::State>, anyhow::Error>
where
    S: EventService + Send + std::marker::Sync + 'static,
    T: Twin + Clone + std::marker::Sync + 'static,
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
    Ok(TwinExecuter::new(rx, 100))
}

#[allow(dead_code)]
pub async fn twin_current_state<S, T>(
    event_service: S,
    twin: T,
) -> Box<Result<T::State, anyhow::Error>>
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
    S: Debug + Unpin + Clone + Send + std::marker::Sync,
{
    type Item = S;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.input.poll_recv(cx) {
            Poll::Ready(Some(state)) => {
                #[allow(unused_unsafe)]
                let this: &mut Self = unsafe { Pin::get_mut(self) };
                this.last_state = Some(state);
                this.last_interaction = Instant::now();
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(None) => return Poll::Ready(self.last_state.clone()),
            _ => (),
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
pub fn observe<T, S, F>(mut twin: T, func: F) -> tokio::task::JoinHandle<()>
where
    F: Fn(S) -> () + Send + 'static,
    S: Debug + Unpin + Clone + Send + std::marker::Sync,
    T: Stream<Item = S> + std::marker::Unpin + Send + 'static,
{
    tokio::spawn(async move {
        'observeLaunchpad: loop {
            match twin.next().await {
                Some(state) => func(state),
                None => break 'observeLaunchpad,
            }
        }
        println!("I'm done here");
    })
}
