use anyhow::Result;
use futures::Stream;
use std::time::Duration;
use tokio::time;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

use crate::text::{Attributes, Text};

use super::{WidgetStreamI, WidgetStream};

/// Shows the current time and date.
///
/// This widget shows the current time and date, in the form `%Y-%m-%d %a %I:%M
/// %p`, e.g. `2017-09-01 Fri 12:51 PM`.
pub struct Clock {
    attr: Attributes,
    format_str: Option<String>,
}

impl Clock {
    // Creates a new Clock widget.
    pub fn new(attr: Attributes, format_str: Option<String>) -> WidgetStream<Self, impl Stream<Item = WidgetStreamI>> {
        WidgetStream::new(
            Self {
                attr, format_str
            },
            Self::into_stream
        )
    }

    fn into_stream(self) -> Result<impl Stream<Item = WidgetStreamI>> {
        // As we're not showing seconds, we can sleep for however long
        // it takes until the minutes changes between updates.
        let one_minute = Duration::from_secs(60);
        let interval = time::interval(one_minute);
        let stream = IntervalStream::new(interval).map(move |_| Ok(self.tick()));

        return Ok(stream)
    }

    fn tick(&self) -> Vec<Text> {
        let now = chrono::Local::now();
        let format_time: String = self
            .format_str
            .clone()
            .map_or("%Y-%m-%d %a %I:%M %p".to_string(), |item| item);
        let text = now.format(&format_time).to_string();
        let texts = vec![Text {
            attr: self.attr.clone(),
            text,
            stretch: false,
            markup: true,
        }];
        texts
    }
}
