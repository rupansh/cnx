//! A simple X11 status bar for use with simple WMs.
//!
//! Cnx is written to be customisable, simple and fast. Where possible, it
//! prefers to asynchronously wait for changes in the underlying data sources
//! (and uses [`tokio`] to achieve this), rather than periodically
//! calling out to external programs.
//!
//! # How to use
//!
//! Cnx is a library that allows you to make your own status bar.
//!
//! In normal usage, you will create a new binary project that relies on the
//! `cnx` crate, and customize it through options passed to the main [`Cnx`]
//! object and its widgets. (It's inspired by [`QTile`] and [`dwm`], in that the
//! configuration is done entirely in code, allowing greater extensibility
//! without needing complex configuration handling).
//!
//! An simple example of a binary using Cnx is:
//!
//! ```no_run
//!
//! use cnx::text::*;
//! use cnx::widgets::*;
//! use cnx::{Cnx, Position};
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     let attr = Attributes {
//!         font: Font::new("Envy Code R 21"),
//!         fg_color: Color::white(),
//!         bg_color: None,
//!         padding: Padding::new(8.0, 8.0, 0.0, 0.0),
//!     };
//!
//!     let mut cnx = Cnx::new(Position::Top);
//!     cnx.add_widget(ActiveWindowTitle::new(attr.clone()));
//!     cnx.add_widget(Clock::new(attr.clone(), None));
//!     cnx.run()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! A more complex example is given in [`cnx-bin/src/main.rs`] alongside the project.
//! (This is the default `[bin]` target for the crate, so you _could_ use it by
//! either executing `cargo run` from the crate root, or even running `cargo
//! install cnx; cnx`. However, neither of these are recommended as options for
//! customizing Cnx are then limited).
//!
//! Before running Cnx, you'll need to make sure your system has the required
//! dependencies, which are described in the [`README`][readme-deps].
//!
//! # Built-in widgets
//!
//! There are currently these widgets available:
//!
//! - [`crate::widgets::ActiveWindowTitle`] — Shows the title ([`EWMH`]'s `_NET_WM_NAME`) for
//!   the currently focused window ([`EWMH`]'s `_NEW_ACTIVE_WINDOW`).
//! - [`crate::widgets::Pager`] — Shows the WM's workspaces/groups, highlighting whichever is
//!   currently active. (Uses [`EWMH`]'s `_NET_DESKTOP_NAMES`,
//!   `_NET_NUMBER_OF_DESKTOPS` and `_NET_CURRENT_DESKTOP`).
//! - [`crate::widgets::Clock`] — Shows the time.
//!
//! The cnx-contrib crate contains additional widgets:
//!
//! - **Sensors** — Periodically parses and displays the output of the
//!   sensors provided by the system.
//! - **Volume** - Shows the current volume/mute status of the default output
//!   device.
//! - **Battery** - Shows the remaining battery and charge status.
//! - **Wireless** - Shows the wireless strength of your current network.
//! - **CPU** - Shows the current CPU consumption
//! - **Weather** - Shows the Weather information of your location
//! - **Disk Usage** - Show the current usage of your monted filesystem
//!
//! The Sensors, Volume and Battery widgets require platform
//! support. They currently support Linux (see dependencies below) and OpenBSD.
//! Support for additional platforms should be possible.
//!
//! # Dependencies
//!
//! In addition to the Rust dependencies in `Cargo.toml`, Cnx also depends on
//! these system libraries:
//!
//!  - `xcb-util`: `xcb-ewmh` / `xcb-icccm` / `xcb-keysyms`
//!  - `x11-xcb`
//!  - `pango`
//!  - `cairo`
//!  - `pangocairo`
//!
//! Some widgets have additional dependencies on Linux:
//!
//!  - **Volume** widget relies on `alsa-lib`
//!  - **Sensors** widget relies on [`lm_sensors`] being installed.
//!  - **Wireless** widget relies on `libiw-dev`.
//!
//! # Creating new widgets
//!
//! Cnx is designed such that thirdparty widgets can be written in
//! external crates and used with the main [`Cnx`] instance. We have
//! [`cnx-contrib`] crate which contains various additional
//! widgets. You can also create new crates or add it to the existing
//! [`cnx-contrib`] crate.
//!
//! The built-in [`widgets`] should give you some examples on which to base
//! your work.
//!
//! [`tokio`]: https://tokio.rs/
//! [`QTile`]: http://www.qtile.org/
//! [`dwm`]: http://dwm.suckless.org/
//! [readme-deps]: https://github.com/mjkillough/cnx/blob/master/README.md#dependencies
//! [`cnx-bin/src/main.rs`]: https://github.com/mjkillough/cnx/blob/master/cnx-bin/src/main.rs
//! [`EWMH`]: https://specifications.freedesktop.org/wm-spec/wm-spec-latest.html
//! [`lm_sensors`]: https://wiki.archlinux.org/index.php/lm_sensors
//! [`cnx-contrib`]: https://github.com/mjkillough/cnx/tree/master/cnx-contrib

#![recursion_limit = "256"]

mod bar;
pub mod text;
pub mod widgets;
mod xcb;

use anyhow::Result;
use futures::Stream;
use tokio_stream::{StreamExt, Empty};
use widgets::{WidgetStreamI, WidgetStream};
use tokio::pin;

use crate::bar::Bar;
use crate::xcb::BarEventStream;

pub use bar::Position;

/// The main object, used to instantiate an instance of Cnx.
///
/// Widgets can be added using the [`add_widget()`] method. Once configured,
/// the [`run()`] method will take ownership of the instance and run it until
/// the process is killed or an error occurs.
///
/// [`add_widget()`]: #method.add_widget
/// [`run()`]: #method.run
pub struct Cnx<FullStream: Stream<Item = (usize, WidgetStreamI)> + 'static> {
    bar: Bar,
    stream: FullStream,
}

impl Cnx<Empty<(usize, WidgetStreamI)>> {
    /// Creates a new `Cnx` instance.
    ///
    /// This creates a new `Cnx` instance at either the top or bottom of the
    /// screen, depending on the value of the [`Position`] enum.
    ///
    /// [`Position`]: enum.Position.html
    pub fn new(position: Position) -> Result<Self> {
        Ok(Self {
            bar: Bar::new(position)?,
            stream: tokio_stream::empty(),
        })
    }
}

impl<FullStream: Stream<Item = (usize, WidgetStreamI)> + 'static> Cnx<FullStream> {
    /// Adds a widget to the `Cnx` instance.
    ///
    /// Takes ownership of the [`Widget`] and adds it to the Cnx instance to
    /// the right of any existing widgets.
    ///
    /// [`Widget`]: widgets/trait.Widget.html
    pub fn add_widget<T: 'static, S: Stream<Item = WidgetStreamI> + 'static>(mut self, stream: WidgetStream<T, S>) -> Result<Cnx<impl Stream<Item = (usize, WidgetStreamI)> + 'static>> {
        let idx = self.bar.add_content(Vec::new())?;
        Ok(Cnx {
            bar: self.bar,
            stream: self.stream.merge(stream.into_stream()?.map(move |v| (idx, v))),
        })
    }


    /// Runs the Cnx instance.
    ///
    /// This method takes ownership of the Cnx instance and runs it until either
    /// the process is terminated, or an internal error is returned.
    pub async fn run(self) -> Result<()> {
        let bar = self.bar;
        let stream = self.stream;

        let mut event_stream = BarEventStream::new(bar)?;
        pin!(stream);
        loop {
            tokio::select! {
                // Pass each XCB event to the Bar.
                Some(event) = event_stream.next() => {
                    if let Err(err) = event_stream.bar_mut().process_event(event) {
                        println!("Error processing XCB event: {}", err);
                    }
                },

                // Each time a widget yields new values, pass to the bar.
                // Ignore (but log) any errors from widgets.
                Some((idx, result)) = stream.next() => {
                    match result {
                        Err(err) => println!("Error from widget {}: {}", idx, err),
                        Ok(texts) => {
                            if let Err(err) = event_stream.bar_mut().update_content(idx, texts) {
                                println!("Error updating widget {}: {}", idx, err);
                            }
                        }
                    }
                }
            }
        }
    }
}
