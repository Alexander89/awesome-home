use core::task::Context;
use futures::StreamExt;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::Poll;

pin_project! {
    pub struct CombineLatest<S, I>
    {
        #[pin]
        streams: Vec<S>,
        #[pin]
        last_state: Vec<Option<I>>,
        #[pin]
        live_mode: bool
    }
}

#[allow(dead_code)]
pub fn combine_latest<S, I>(streams: Vec<S>) -> CombineLatest<S, I>
where
    S: tokio_stream::Stream<Item = I>,
    I: Clone,
{
    CombineLatest {
        last_state: vec![None; streams.len()],
        streams,
        live_mode: true,
    }
}

impl<S, I> tokio_stream::Stream for CombineLatest<S, I>
where
    S: tokio_stream::Stream<Item = I> + std::marker::Unpin,
    I: Clone,
{
    type Item = Vec<I>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut self_proj = self.project();
        let mut at_least_one_updated = false;

        for (idx, stream) in self_proj.streams.iter_mut().enumerate() {
            'stateCollectLoop: while let Poll::Ready(p) = stream.poll_next_unpin(cx) {
                if let Some(state) = p {
                    let mut l = self_proj.last_state.clone();
                    l[idx] = Some(state);
                    self_proj.last_state.set(l);

                    at_least_one_updated = true;
                    if *self_proj.live_mode == false {
                        self_proj
                            .live_mode
                            .set(self_proj.last_state.iter().all(|s| s.is_some()));
                    }
                } else {
                    break 'stateCollectLoop;
                }
            }
        }

        if *self_proj.live_mode == true && at_least_one_updated {
            Poll::Ready(Some(
                self_proj
                    .last_state
                    .iter()
                    .map(|s| s.clone().unwrap())
                    .collect(),
            ))
        } else {
            Poll::Pending
        }
    }
}
