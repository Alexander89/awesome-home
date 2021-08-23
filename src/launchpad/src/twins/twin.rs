use actyx_sdk::{
    language::Query,
    service::{
        EventResponse, EventService, SessionId, StartFrom, SubscribeMonotonicRequest,
        SubscribeMonotonicResponse,
    },
    OffsetMap, Payload,
};
use futures::StreamExt;
use std::{
    sync::mpsc::{self, Receiver},
    thread::sleep,
    time::Duration,
};

pub trait Twin: Clone + Send {
    type State: Default + Clone + Send;

    fn name(&self) -> String;
    fn id(&self) -> String;
    fn query(&self) -> Query;

    fn reducer(state: Self::State, event: EventResponse<Payload>) -> Self::State;
}

pub fn execute_twin<S, T>(event_service: S, twin: T) -> Result<Receiver<T::State>, anyhow::Error>
where
    T: Twin + Clone + std::marker::Sync + 'static,
    S: EventService + Send + std::marker::Sync + 'static,
{
    let (tx, rx) = mpsc::channel::<T::State>();
    tokio::spawn(async move {
        let launchpad_subscription = event_service
            .subscribe_monotonic(SubscribeMonotonicRequest {
                session: SessionId::from(format!("{}:{}", twin.name(), twin.id())),
                from: StartFrom::LowerBound(OffsetMap::empty()),
                query: twin.query(),
            })
            .await;

        println!("connected");

        let mut state: T::State = Default::default();
        tx.send(state.clone()).expect("no more listeners?");
        if let Ok(mut stream) = launchpad_subscription {
            println!("sub ok");
            loop {
                match stream.next().await {
                    Some(SubscribeMonotonicResponse::Event { event, caught_up }) => {
                        println!("event");
                        state = T::reducer(state, event);
                        println!("caught_up {}", caught_up);
                        if !caught_up {
                            tx.send(state.clone()).expect("no more listeners?");
                        }
                    }
                    Some(SubscribeMonotonicResponse::Offsets(..)) => {
                        tx.send(state.clone()).expect("no more listeners?");
                    }
                    None => {
                        println!("no event");
                        sleep(Duration::from_millis(100));
                    }
                    Some(other) => {
                        println!("event {:?} ", other);
                    }
                }
            }
        }
        // Process each socket concurrently.
    });

    Ok(rx)
}
