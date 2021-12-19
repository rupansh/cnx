//! Provided widgets and types for creating new widgets.

mod active_window_title;

mod clock;
mod pager;
pub use self::active_window_title::ActiveWindowTitle;
pub use self::clock::Clock;
pub use self::pager::Pager;
use crate::text::Text;
use anyhow::Result;
use futures::stream::Stream;

pub type WidgetStreamI = Result<Vec<Text>>;

pub struct WidgetStream<T, S: Stream<Item = WidgetStreamI>> {
    widget: T,
    stream_gen: fn(T) -> Result<S>
}

impl<T: 'static, S: Stream<Item= WidgetStreamI> + 'static> WidgetStream<T, S> {
    pub fn new(widget: T, stream_gen: fn(T) -> Result<S>) -> Self {
        return Self {
            widget,
            stream_gen
        }
    }

    pub(crate) fn into_stream(self: Self) -> Result<S> {
        return (self.stream_gen)(self.widget);
    }
}
