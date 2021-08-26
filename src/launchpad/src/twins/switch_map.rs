use core::task::Context;
use futures::StreamExt;
use pin_project_lite::pin_project;
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
pub fn switch_map<I, T, O>(from: I, mapper: T) -> SwitchMap<I, T, O>
where
    I: Stream,
    T: Fn(I::Item) -> Option<O> + Clone,
    O: Stream,
{
    SwitchMap {
        from: from,
        mapper,
        mapped_stream: None,
    }
}

impl<I, T, O> Stream for SwitchMap<I, T, O>
where
    I: Stream,
    T: Fn(I::Item) -> Option<O> + Clone,
    O: Stream,
{
    type Item = O::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut self_proj = self.project();
        while let Poll::Ready(p) = self_proj.from.poll_next_unpin(cx) {
            if let Some(state) = p {
                self_proj.mapped_stream.set((self_proj.mapper)(state));
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
