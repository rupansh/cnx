use anyhow::Result;
use byte_unit::{Byte, ByteUnit};
use cnx::text::{Attributes, Text};
use cnx::widgets::{WidgetStream, WidgetStreamI};
use nix::sys::statvfs::statvfs;
use std::time::Duration;
use tokio::time;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::{StreamExt, Stream};

/// Represent Information about the mounted filesystem
#[derive(Debug)]
pub struct DiskInfo {
    /// Total size of the filesystem
    pub total: Byte,
    /// Total used space of the filesystem
    pub used: Byte,
    /// Total free space of the filesystem
    pub free: Byte,
}

impl DiskInfo {
    fn new(path: &str) -> Result<Self> {
        let stat = statvfs(path)?;
        let total_size = stat.blocks() * stat.fragment_size();
        let used = (stat.blocks() - stat.blocks_free()) * stat.fragment_size();
        let available = stat.blocks_available() * stat.fragment_size();
        let total = byte_unit::Byte::from_bytes(total_size as u128);
        let used = byte_unit::Byte::from_bytes(used as u128);
        let free: Byte = byte_unit::Byte::from_bytes(available as u128);

        let disk_info = DiskInfo { total, used, free };
        Ok(disk_info)
    }
}

/// Disk usage widget to show current usage and remaining free space
/// in the mounted filesystem.
pub struct DiskUsage<F: Fn(DiskInfo) -> String> {
    attr: Attributes,
    path: String,
    render: F,
}

fn default_render(info: DiskInfo) -> String {
    format!(
        "Disk: {}/{}",
        info.used.get_adjusted_unit(ByteUnit::GiB).format(0),
        info.total.get_adjusted_unit(ByteUnit::GiB).format(0)
    )
}

impl DiskUsage<fn(DiskInfo) -> String> {
    pub fn new(attr: Attributes, path: String) -> WidgetStream<Self, impl Stream<Item = WidgetStreamI>> {
        WidgetStream::new(
            Self {
                attr,
                path,
                render: default_render
            },
            Self::into_stream
        )
    }
}

impl<F: Fn(DiskInfo) -> String + 'static> DiskUsage<F> {
    /// Creates a new [`DiskUsage`] widget.
    ///
    /// Arguments
    ///
    /// * `attr` - Represents `Attributes` which controls properties like
    /// `Font`, foreground and background color etc.
    ///
    /// * `path` - Pathname of any file within the mounted filesystem.

    /// * `render` - We use the closure to control the way output is
    /// displayed in the bar. [`DiskInfo`] represents the details
    /// about the mounted filesystem.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate cnx;
    /// #
    /// # use cnx::*;
    /// # use cnx::text::*;
    /// # use cnx_contrib::widgets::disk_usage::*;
    /// # use anyhow::Result;
    /// #
    /// # fn run() -> Result<()> {
    /// let attr = Attributes {
    ///     font: Font::new("SourceCodePro 21"),
    ///     fg_color: Color::white(),
    ///     bg_color: None,
    ///     padding: Padding::new(8.0, 8.0, 0.0, 0.0),
    /// };
    ///
    /// let mut cnx = Cnx::new(Position::Top);
    /// cnx.add_widget(DiskUsage::new(attr, "/home".into(), None));
    /// # Ok(())
    /// # }
    /// # fn main() { run().unwrap(); }
    /// ```
    pub fn new_with_render(
        attr: Attributes,
        path: String,
        render: F,
    ) -> WidgetStream<Self, impl Stream<Item = WidgetStreamI>> {
        WidgetStream::new(
            Self { attr, render, path },
            Self::into_stream
        )
    }

    fn tick(&self) -> Result<Vec<Text>> {
        let disk_info = DiskInfo::new(self.path.as_ref())?;

        let text: String = (self.render)(disk_info);
        let texts = vec![Text {
            attr: self.attr.clone(),
            text,
            stretch: false,
            markup: true,
        }];
        Ok(texts)
    }

    fn into_stream(self) -> Result<impl Stream<Item = WidgetStreamI>> {
        let one_hour = Duration::from_secs(3600);
        let interval = time::interval(one_hour);
        let stream = IntervalStream::new(interval).map(move |_| self.tick());

        Ok(stream)
    }
}
