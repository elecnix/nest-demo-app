//! The demo app: every part of the app API, on one screen.
//!
//! Five modes, one wheel, and the whole scene API:
//!
//! | Mode | What it shows |
//! |---|---|
//! | COUNTER | the wheel's own count, in the largest text that fits |
//! | DIAL | a 0-to-100 gauge: a stroked track, a fill with round caps, ticks around it |
//! | TEXT | a typography sample: sizes, weights, tracking, all three anchors, a quadratic curve |
//! | ICONS | Material Design Icons by name, one picked by the wheel, with a ring the app built |
//! | DATA | the last payload Home Assistant pushed, pretty-printed, wrapped, paged and dated |
//!
//! Every wheel step moves the demo one position: the mode advances, and the current mode's own
//! value moves with it — the counter counts, the dial fills, the tracking widens, the icon
//! highlight walks, the payload turns a page. A step that lands in a new mode plays the click and
//! sends Home Assistant an `app_event`. The position is saved in `data/state.json`, so reopening
//! the card comes back to the same mode.
//!
//! The loop runs with a one-second deadline so the data mode can age the payload it is showing:
//! [`Demo::age_changed`] only says yes once per new second, so a tick that changes nothing on
//! screen costs nothing but a read timeout.

mod demo;
mod screen;
mod text;

use std::time::{Duration, SystemTime};

use nest_app_sdk::{App, Handler};
use serde_json::Value;

use crate::demo::{DATA, Demo};

/// One second: fine enough for a payload's age, slow enough to leave the Nest alone.
const PERIOD: Duration = Duration::from_secs(1);

/// The app, as the Nest drives it.
struct DemoApp {
    demo: Demo,
}

impl Handler for DemoApp {
    fn on_open(&mut self, app: &mut App) {
        app.send_scene(&screen::screen(&self.demo));
    }

    fn on_wheel(&mut self, app: &mut App, steps: i32) {
        let turn = self.demo.wheel(steps);
        if !turn.moved {
            return;
        }
        app.send_scene(&screen::screen(&self.demo));
        if turn.mode_changed {
            app.tick();
            app.event(&self.demo.event());
        }
    }

    fn on_data(&mut self, app: &mut App, data: &Value) {
        self.demo.arrived(data);
        if self.demo.mode() == DATA {
            app.send_scene(&screen::screen(&self.demo));
        }
    }

    fn on_tick(&mut self, app: &mut App) {
        if self.demo.mode() == DATA && self.demo.age_changed(SystemTime::now()) {
            app.send_scene(&screen::screen(&self.demo));
        }
    }
}

fn main() {
    let mut app = App::connect();
    let mut demo = DemoApp { demo: Demo::load() };
    if let Err(e) = app.run_with_timeout(Some(PERIOD), &mut demo) {
        eprintln!("demo: the Nest is gone: {e}");
        std::process::exit(1);
    }
}
