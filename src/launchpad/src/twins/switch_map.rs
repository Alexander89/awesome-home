use core::task::Context;
use futures::StreamExt;
use pin_project_lite::pin_project;
use std::fmt::Debug;
use std::pin::Pin;
use std::task::Poll;
use tokio_stream::Stream;

pin_project! {
    pub struct SwitchMap<I, T, O>
    {
        #[pin]
        from: I,
        mapper: T,
        #[pin]
        mapped_stream: Option<O>,
    }
}

#[allow(dead_code)]
pub fn switch_map<I, T, S, O, OI>(from: I, mapper: T) -> SwitchMap<I, T, O>
where
    I: Stream<Item = S>,
    T: Fn(S) -> O + Clone,
    O: Stream<Item = OI>,
{
    SwitchMap {
        from: from,
        mapper,
        mapped_stream: None,
    }
}

impl<I, T, S, O, OI> Stream for SwitchMap<I, T, O>
where
    I: Stream<Item = S> + Unpin,
    T: Fn(S) -> O + Clone,
    O: Stream<Item = OI> + Debug,
{
    type Item = OI;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut self_proj = self.project();
        while let Poll::Ready(p) = self_proj.from.poll_next_unpin(cx) {
            if let Some(state) = p {
                self_proj.mapped_stream.set(Some((self_proj.mapper)(state)));
            } else {
                return Poll::Ready(None);
            }
        }
        if let Some(mut mapped) = self_proj.mapped_stream.as_mut().as_pin_mut() {
            while let Poll::Ready(p) = mapped.poll_next_unpin(cx) {
                if let Some(state) = p {
                    return Poll::Ready(Some(state));
                } else {
                    self_proj.mapped_stream.set(None);
                    return Poll::Pending;
                }
            }
        }

        Poll::Pending
    }
}
