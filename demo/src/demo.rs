//! The demo's state: one wheel-driven number, and everything else derived from it.
//!
//! The wheel is the only input an app gets, so the demo leans on it: every detent moves one
//! position, and the mode, each mode's value and the mode's page are that position reduced. Turn
//! clockwise and the demo walks forward through the five modes; turn back and it walks the other
//! way. Nothing is stored but the position, and the mode is read back from it, which is why the
//! demo reopens on the screen it was left on.
//!
//! ```
//! use nest_app_sdk::Store;
//! let mut demo = nest_app_demo::Demo::load_from(Store::at("/nonexistent"));
//! assert_eq!(demo.mode(), nest_app_demo::COUNTER);
//! let moved = demo.wheel(1);
//! assert!(moved.moved && moved.mode_changed);
//! assert_eq!(demo.counter(), 1);
//! ```

use std::time::SystemTime;

use nest_app_sdk::Store;
use serde_json::{Value, json};

/// The five modes, in wheel order.
pub const MODE_NAMES: [&str; 5] = ["COUNTER", "DIAL", "TEXT", "ICONS", "DATA"];

/// How many modes there are.
pub const MODE_COUNT: usize = MODE_NAMES.len();

/// The big-number mode.
pub const COUNTER: usize = 0;
/// The gauge mode.
pub const DIAL: usize = 1;
/// The typography mode.
pub const TEXT: usize = 2;
/// The icon-grid mode.
pub const ICONS: usize = 3;
/// The Home Assistant payload mode.
pub const DATA: usize = 4;

/// The file the position is kept in, under `data/`.
pub const STATE: &str = "state.json";

/// The dial is 0 to 100.
const DIAL_RANGE: i64 = 101;

/// What one wheel message did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wheel {
    /// The wheel moved: the scene needs redrawing.
    pub moved: bool,
    /// It moved into another mode: play the click and tell Home Assistant.
    pub mode_changed: bool,
}

/// The demo's whole state: where the wheel is, and the last payload Home Assistant sent.
pub struct Demo {
    /// The wheel position. Every mode, value and page is this number reduced.
    position: i64,
    data: Option<Value>,
    data_at: Option<SystemTime>,
    /// The payload's age as last drawn, so a tick only redraws when the seconds change.
    drawn_age: Option<u64>,
    store: Store,
    save_warned: bool,
}

impl Demo {
    /// Load the position from `data/state.json`, or start at zero.
    pub fn load() -> Self {
        Self::load_from(Store::new())
    }

    /// Load from a particular store — for tests, and for an app embedded elsewhere.
    pub fn load_from(store: Store) -> Self {
        let position = store.load(STATE).as_ref().and_then(position_of).unwrap_or_default();
        Self {
            position,
            data: None,
            data_at: None,
            drawn_age: None,
            store,
            save_warned: false,
        }
    }

    /// The wheel position, as the counter shows it.
    pub fn position(&self) -> i64 {
        self.position
    }

    /// The current mode, 0 to [`MODE_COUNT`] - 1.
    pub fn mode(&self) -> usize {
        self.pick(MODE_COUNT)
    }

    /// The counter's value: every wheel step, counted.
    pub fn counter(&self) -> i64 {
        self.position
    }

    /// The dial's value, 0 to 100.
    pub fn dial(&self) -> i64 {
        self.position.rem_euclid(DIAL_RANGE)
    }

    /// The wheel's position modulo `n`, for whatever a mode pages or picks through.
    ///
    /// `rem_euclid` rather than `%`, so turning the wheel backwards stays in range.
    pub fn pick(&self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            self.position.rem_euclid(n as i64) as usize
        }
    }

    /// Turn the wheel. Saves the new position when it moved.
    pub fn wheel(&mut self, steps: i32) -> Wheel {
        let was = self.mode();
        self.position = self.position.saturating_add(i64::from(steps));
        let moved = steps != 0;
        if moved {
            self.save();
        }
        Wheel {
            moved,
            mode_changed: moved && self.mode() != was,
        }
    }

    /// Remember a payload from Home Assistant.
    pub fn arrived(&mut self, data: &Value) {
        self.arrived_at(data, SystemTime::now());
    }

    /// The same, with the arrival time given — for tests that want a readable clock.
    pub fn arrived_at(&mut self, data: &Value, at: SystemTime) {
        self.data = Some(data.clone());
        self.data_at = Some(at);
        self.drawn_age = None;
    }

    /// The last payload, if Home Assistant has sent one.
    pub fn data(&self) -> Option<&Value> {
        self.data.as_ref()
    }

    /// When the last payload arrived.
    pub fn data_at(&self) -> Option<SystemTime> {
        self.data_at
    }

    /// How many seconds ago the last payload arrived.
    pub fn age(&self, now: SystemTime) -> Option<u64> {
        let at = self.data_at?;
        now.duration_since(at).ok().map(|d| d.as_secs())
    }

    /// True the first time it is asked about a given second of the payload's age.
    ///
    /// That is what keeps a ticking app from redrawing once a second forever: the tick handler
    /// asks, and only the ticks that land on a new second get a redraw out of it.
    pub fn age_changed(&mut self, now: SystemTime) -> bool {
        let Some(age) = self.age(now) else {
            return false;
        };
        if self.drawn_age == Some(age) {
            return false;
        }
        self.drawn_age = Some(age);
        true
    }

    /// The scene line for this state — what Home Assistant sees in an `app_event`.
    pub fn event(&self) -> Value {
        json!({
            "mode": self.mode(),
            "name": MODE_NAMES[self.mode()],
            "position": self.position(),
            "value": match self.mode() {
                DIAL => self.dial(),
                _ => self.counter(),
            },
        })
    }

    fn save(&mut self) {
        if let Err(e) = self.store.save(STATE, &json!({ "position": self.position }))
            && !self.save_warned
        {
            eprintln!("demo: cannot save {}: {e}", self.store.path(STATE).display());
            self.save_warned = true;
        }
    }
}

/// The position in a loaded state document, tolerating whatever is in there.
fn position_of(state: &Value) -> Option<i64> {
    let n = state.get("position")?;
    if let Some(i) = n.as_i64() {
        return Some(i);
    }
    n.as_f64().map(|f| f as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, UNIX_EPOCH};

    fn temp(name: &str) -> (Store, PathBuf) {
        let dir = std::env::temp_dir().join(format!("nest-app-demo-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        (Store::at(&dir), dir)
    }

    #[test]
    fn the_wheel_walks_the_modes_and_moves_every_value() {
        let (store, dir) = temp("wheel");
        let mut demo = Demo::load_from(store);
        assert_eq!((demo.mode(), demo.counter(), demo.dial()), (COUNTER, 0, 0));

        for expected in [DIAL, TEXT, ICONS, DATA, COUNTER] {
            let w = demo.wheel(1);
            assert!(w.moved && w.mode_changed, "one step, one mode");
            assert_eq!(demo.mode(), expected);
        }
        assert_eq!(demo.counter(), 5, "the counter counts wheel steps");
        assert_eq!(demo.dial(), 5, "and the dial follows it");

        // Backwards, and the mode walks back.
        let w = demo.wheel(-1);
        assert!(w.mode_changed);
        assert_eq!(demo.mode(), DATA, "one turn back is the last mode");
        assert_eq!(demo.counter(), 4);

        // A whole lap is a value change without a mode change.
        let w = demo.wheel(MODE_COUNT as i32);
        assert!(w.moved && !w.mode_changed);
        assert_eq!(demo.mode(), DATA);
        assert_eq!(demo.counter(), 9);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_zero_step_does_nothing() {
        let mut demo = Demo::load_from(Store::at("/nonexistent"));
        assert_eq!(
            demo.wheel(0),
            Wheel {
                moved: false,
                mode_changed: false
            }
        );
    }

    #[test]
    fn values_stay_in_range_forwards_and_backwards() {
        let (store, dir) = temp("range");
        let mut demo = Demo::load_from(store);
        demo.wheel(-1);
        assert_eq!(demo.dial(), 100, "0 - 1 wraps to the top of the dial");
        assert_eq!(demo.mode(), DATA);
        demo.wheel(-10);
        assert_eq!(demo.mode(), DATA, "-11 reduced is 4: the wheel is a ring");
        assert_eq!(demo.pick(7), 3, "pick() stays in range too");
        assert_eq!(demo.pick(0), 0);
        for _ in 0..200 {
            demo.wheel(1);
            assert!((0..=100).contains(&demo.dial()));
            assert!(demo.mode() < MODE_COUNT);
            assert!(demo.pick(8) < 8);
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_position_survives_a_restart() {
        let (store, dir) = temp("persist");
        let mut demo = Demo::load_from(store.clone());
        demo.wheel(3);
        demo.wheel(1);
        let before = (demo.position(), demo.mode());
        let reopened = Demo::load_from(store);
        assert_eq!((reopened.position(), reopened.mode()), before);
        assert_eq!(before, (4, DATA));
        assert_eq!(
            reopen_state(&dir),
            json!({"position": 4}),
            "one number, in data/state.json"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    fn reopen_state(dir: &Path) -> Value {
        let text = std::fs::read_to_string(dir.join(STATE)).expect("state.json");
        serde_json::from_str(&text).expect("JSON")
    }

    #[test]
    fn a_missing_or_nonsense_state_file_starts_at_zero() {
        let (store, dir) = temp("corrupt");
        assert_eq!(Demo::load_from(store.clone()).position(), 0);
        std::fs::create_dir_all(&dir).expect("mkdir");
        for text in ["", "{", "null", "[]", "{\"position\": \"four\"}", "{\"mode\": 3}"] {
            std::fs::write(dir.join(STATE), text).expect("write");
            assert_eq!(Demo::load_from(store.clone()).position(), 0, "{text:?}");
        }
        // A float, though, is read as best it can be.
        std::fs::write(dir.join(STATE), "{\"position\": 7.0}").expect("write");
        assert_eq!(Demo::load_from(store.clone()).position(), 7);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_payload_is_remembered_with_its_arrival_and_age() {
        let (store, dir) = temp("age");
        let mut demo = Demo::load_from(store);
        assert_eq!(demo.data(), None);
        assert_eq!(demo.age(SystemTime::now()), None);
        assert!(!demo.age_changed(SystemTime::now()), "nothing to age yet");

        let at = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        demo.arrived_at(&json!({"stop": "Gare Centrale"}), at);
        assert_eq!(demo.data(), Some(&json!({"stop": "Gare Centrale"})));
        assert_eq!(demo.data_at(), Some(at));
        assert_eq!(demo.age(at + Duration::from_secs(9)), Some(9));
        assert_eq!(demo.age(at - Duration::from_secs(1)), None, "before it arrived");

        // The tick only redraws on a new second.
        assert!(demo.age_changed(at + Duration::from_secs(1)));
        assert!(!demo.age_changed(at + Duration::from_secs(1)));
        assert!(!demo.age_changed(at + Duration::from_millis(1_400)));
        assert!(demo.age_changed(at + Duration::from_secs(2)));
        // A new payload resets the clock.
        demo.arrived_at(&json!(1), at);
        assert!(demo.age_changed(at));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_event_says_which_mode_and_what_value() {
        let (store, dir) = temp("event");
        let mut demo = Demo::load_from(store);
        assert_eq!(
            demo.event(),
            json!({"mode": 0, "name": "COUNTER", "position": 0, "value": 0})
        );
        demo.wheel(2);
        assert_eq!(
            demo.event(),
            json!({"mode": 2, "name": "TEXT", "position": 2, "value": 2})
        );
        demo.wheel(1);
        assert_eq!(demo.event()["value"], 3);
        let _ = std::fs::remove_dir_all(dir);
    }
}
