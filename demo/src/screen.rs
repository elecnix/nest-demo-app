//! What each mode draws.
//!
//! Every item here comes from the SDK's builders, so the demo is also the worked example of the
//! whole scene API: filled paths, stroked arcs with round caps, the mode ring's annulus segments,
//! text at every weight, size, anchor and tracking, Material Design Icons by name, and a quadratic
//! curve. The layout keeps to a disc of radius ~150 px around (160, 160), inside the display's
//! ~163 px rim, so nothing is clipped by the glass.

use std::time::SystemTime;

use nest_app_sdk::{
    path,
    scene::{self, Anchor, TextStyle},
};
use serde_json::Value;

use crate::demo::{COUNTER, DATA, DIAL, Demo, ICONS, MODE_COUNT, MODE_NAMES, TEXT};
use crate::text::LINE_CHARS;
use crate::text::{self, MAX_LINES};

/// The canvas centre, and the centre of the disc the screen is.
const C: f32 = 160.0;
/// The teal the Nest's own entity discs are filled with.
const TEAL: &str = "#7cd6c9";
/// Text that has to be read.
const WHITE: &str = "#ffffff";
/// The small print, the inactive ring segments and the rules.
const DIM: &str = "#2f5b56";
/// The fill behind the picked icon cell.
const DEEP: &str = "#14312e";
/// The mode ring's radius: inside the 163 px rim, clear of the corner content.
const RING: f32 = 150.0;
/// Each mode's share of the ring, with a gap between neighbours.
const RING_SPAN: f32 = 56.0;
const RING_GAP: f32 = 72.0;

/// The icons the device's table knows, drawn in the icon mode.
const ICON_GRID: [&str; 7] = ["lightbulb", "fan", "gauge", "palette", "shape", "thermometer", "eye"];

/// The text mode's letter tracking, one value per wheel position.
const TRACKING: [f32; 3] = [0.0, 1.5, 3.0];

/// The whole scene for the demo's current state.
pub fn screen(demo: &Demo) -> Vec<Value> {
    screen_at(demo, SystemTime::now())
}

/// The same, with the clock given — the data mode shows how long ago the payload arrived.
pub fn screen_at(demo: &Demo, now: SystemTime) -> Vec<Value> {
    let mut items = Vec::new();
    chrome(&mut items, demo.mode());
    match demo.mode() {
        COUNTER => counter(&mut items, demo),
        DIAL => dial(&mut items, demo),
        TEXT => typography(&mut items, demo),
        ICONS => icon_grid(&mut items, demo),
        DATA => payload(&mut items, demo, now),
        _ => {}
    }
    items
}

/// The header and the mode ring, on every screen.
fn chrome(items: &mut Vec<Value>, mode: usize) {
    items.push(scene::text(
        MODE_NAMES[mode],
        C,
        46.0,
        15.0,
        TEAL,
        TextStyle::new().bold().tracking(2.4),
    ));
    for i in 0..MODE_COUNT {
        let start = i as f32 * RING_GAP + 8.0;
        let lit = i == mode;
        items.push(scene::stroke(
            path::arc(C, C, RING, start, start + RING_SPAN),
            if lit { TEAL } else { DIM },
            if lit { 6.0 } else { 3.0 },
            true,
        ));
    }
}

/// Mode 1: the wheel's own count, as big as it can be.
fn counter(items: &mut Vec<Value>, demo: &Demo) {
    let value = demo.counter().to_string();
    let size = match value.len() {
        0..=3 => 96.0,
        4 => 76.0,
        5 => 64.0,
        6 => 54.0,
        _ => 44.0,
    };
    items.push(scene::text(
        &value,
        C,
        C + 0.35 * size,
        size,
        WHITE,
        TextStyle::new().bold(),
    ));
    items.push(scene::text(
        "WHEEL STEPS",
        C,
        236.0,
        12.0,
        DIM,
        TextStyle::new().tracking(2.4),
    ));
}

/// Mode 2: a gauge over 0 to 100, filled as the wheel turns.
fn dial(items: &mut Vec<Value>, demo: &Demo) {
    let value = demo.dial();
    let (radius, width) = (104.0, 14.0);
    // The Nest's own dial opens at 225° and sweeps 270° clockwise, which is the gap at the bottom.
    let (start, sweep) = (225.0, 270.0);
    items.push(scene::stroke(
        path::arc(C, C, radius, start, start + sweep),
        DIM,
        width,
        true,
    ));
    if value > 0 {
        let end = start + sweep * value as f32 / 100.0;
        items.push(scene::stroke(path::arc(C, C, radius, start, end), TEAL, width, true));
    }
    // The ticks the Nest's dial has, one every 10 of its 100.
    for i in 0..=10 {
        let angle = start + sweep * i as f32 / 10.0;
        let (x0, y0) = path::point_at(C, C, 76.0, angle);
        let (x1, y1) = path::point_at(C, C, if i % 5 == 0 { 88.0 } else { 84.0 }, angle);
        items.push(scene::stroke(
            path::line(x0, y0, x1, y1),
            DIM,
            if i % 5 == 0 { 2.5 } else { 1.5 },
            true,
        ));
    }
    items.push(scene::text(
        &value.to_string(),
        C,
        C + 15.0,
        44.0,
        WHITE,
        TextStyle::new().bold(),
    ));
    for (label, angle) in [("0", start), ("100", start + sweep)] {
        let (x, y) = path::point_at(C, C, 128.0, angle);
        items.push(scene::text(label, x, y + 5.0, 12.0, DIM, TextStyle::new()));
    }
}

/// Mode 3: a typography sample — sizes, weights, tracking, anchors, and a quadratic curve.
fn typography(items: &mut Vec<Value>, demo: &Demo) {
    let tracking = TRACKING[demo.pick(TRACKING.len())];
    items.push(scene::text(
        "Aa Gg Qq",
        30.0,
        110.0,
        30.0,
        WHITE,
        TextStyle::new().bold().anchor(Anchor::Start),
    ));
    items.push(scene::text(
        "Regular 13 px",
        30.0,
        134.0,
        13.0,
        WHITE,
        TextStyle::new().anchor(Anchor::Start),
    ));
    items.push(scene::text(
        "Bold 13 px",
        30.0,
        158.0,
        13.0,
        WHITE,
        TextStyle::new().bold().anchor(Anchor::Start),
    ));
    items.push(scene::text(
        "TRACKING",
        30.0,
        182.0,
        13.0,
        TEAL,
        TextStyle::new().anchor(Anchor::Start).tracking(tracking),
    ));
    // The three anchors, with a guide line each so the anchoring is visible.
    for x in [30.0, C, 290.0] {
        items.push(scene::stroke(path::line(x, 198.0, x, 222.0), DIM, 1.0, false));
    }
    items.push(scene::text(
        "start",
        30.0,
        216.0,
        11.0,
        DIM,
        TextStyle::new().anchor(Anchor::Start),
    ));
    items.push(scene::text("middle", C, 216.0, 11.0, DIM, TextStyle::new()));
    items.push(scene::text(
        "end",
        290.0,
        216.0,
        11.0,
        DIM,
        TextStyle::new().anchor(Anchor::End),
    ));
    // A curve drawn with Q rather than with cubics: the fifth path command.
    items.push(scene::stroke(
        path::quad(60.0, 256.0, C, 284.0, 260.0, 256.0),
        TEAL,
        3.0,
        true,
    ));
}

/// Mode 4: the icons the device knows, one picked by the wheel, and the two shapes beside them.
fn icon_grid(items: &mut Vec<Value>, demo: &Demo) {
    let cells = [
        (62.0, 128.0),
        (122.0, 128.0),
        (182.0, 128.0),
        (242.0, 128.0),
        (62.0, 216.0),
        (122.0, 216.0),
        (182.0, 216.0),
        (242.0, 216.0),
    ];
    let picked = demo.pick(cells.len());
    for (i, &(x, y)) in cells.iter().enumerate() {
        let lit = i == picked;
        let color = if lit { TEAL } else { DIM };
        if lit {
            items.push(scene::fill(
                path::rounded_rect(x - 27.0, y - 26.0, 54.0, 62.0, 13.0),
                DEEP,
            ));
        }
        match ICON_GRID.get(i) {
            Some(name) => {
                items.push(scene::icon(&format!("mdi-{name}"), x, y, 34.0, color));
                items.push(scene::text(
                    &name.to_uppercase(),
                    x,
                    y + 28.0,
                    7.5,
                    color,
                    TextStyle::new().tracking(0.6),
                ));
            }
            None => {
                // The eighth cell is not an icon at all: a ring the app built itself, out of an
                // arc and a stroke width.
                items.push(scene::stroke(path::circle(x, y, 14.0), color, 3.5, true));
                items.push(scene::text(
                    "RING",
                    x,
                    y + 28.0,
                    7.5,
                    color,
                    TextStyle::new().tracking(0.6),
                ));
            }
        }
    }
}

/// Mode 5: the last payload Home Assistant pushed, wrapped, paged and dated.
fn payload(items: &mut Vec<Value>, demo: &Demo, now: SystemTime) {
    let Some(value) = demo.data() else {
        items.push(scene::icon("mdi-eye", C, 138.0, 34.0, DIM));
        items.push(scene::text(
            "NO DATA YET",
            C,
            186.0,
            14.0,
            DIM,
            TextStyle::new().tracking(1.5),
        ));
        items.push(scene::text(
            "Home Assistant pushes app_data",
            C,
            208.0,
            9.5,
            DIM,
            TextStyle::new(),
        ));
        return;
    };
    let bytes = serde_json::to_string(value).map(|s| s.len()).unwrap_or(0);
    let arrived = demo.data_at().map(text::clock).unwrap_or_else(|| "??".to_string());
    let age = demo.age(now).unwrap_or(0);
    items.push(scene::text(
        &format!("{bytes} BYTES - {arrived} - {age}S AGO"),
        C,
        76.0,
        10.0,
        TEAL,
        TextStyle::new().tracking(0.8),
    ));

    let lines = text::json_lines(value, LINE_CHARS, MAX_LINES);
    let total = text::pages(lines.len());
    let page = demo.pick(total);
    let (drawn, _) = text::page(&lines, page);
    for (i, line) in drawn.iter().enumerate() {
        items.push(scene::text(
            line,
            32.0,
            96.0 + i as f32 * 13.0,
            9.5,
            WHITE,
            TextStyle::new().anchor(Anchor::Start),
        ));
    }
    if total > 1 {
        items.push(scene::text(
            &format!("{}/{}", page + 1, total),
            C,
            292.0,
            10.0,
            DIM,
            TextStyle::new(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_app_sdk::{MAX_ITEMS, MAX_STROKE, MAX_TEXT, Store};
    use serde_json::json;
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    /// A scratch directory the demo can save into, cleaned up when the test ends.
    ///
    /// The demo writes as the wheel turns, so a test that moves the wheel from a directory that
    /// does not exist would print a save failure — and a shared one, like `/tmp/state.json`, would
    /// leak a position from one test into the next.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("nest-app-demo-screen-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            Self(dir)
        }

        fn demo(&self, position: i64) -> Demo {
            // Each demo starts from a clean file: the previous one saved its own position, and a
            // test that asked for position 5 twice would otherwise get 10.
            let _ = std::fs::remove_file(self.0.join(crate::demo::STATE));
            let mut demo = Demo::load_from(Store::at(&self.0));
            demo.wheel(position as i32);
            demo
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A scene is what the docs promise: at most 256 items, each a known kind, in a colour the
    /// wire takes, with the fields that kind has.
    fn check(items: &[Value]) {
        assert!(!items.is_empty(), "a scene with nothing in it is a black screen");
        assert!(items.len() <= MAX_ITEMS, "{} items", items.len());
        for item in items {
            let k = item["k"].as_str().unwrap_or_else(|| panic!("no kind in {item}"));
            let color = item["c"].as_str().unwrap_or_else(|| panic!("no colour in {item}"));
            assert_eq!(color.len(), 7, "{color} is not #rrggbb");
            assert!(color.starts_with('#') && color[1..].chars().all(|c| c.is_ascii_hexdigit()));
            match k {
                "fill" | "stroke" => {
                    let d = item["d"].as_str().expect("path data");
                    assert!(d.starts_with('M'), "{d} does not start with a move");
                    assert!(
                        d.chars()
                            .all(|c| matches!(c, 'M' | 'L' | 'Q' | 'C' | 'Z' | ' ' | '.' | '-' | '0'..='9')),
                        "path data outside the parser's alphabet: {d}"
                    );
                    if k == "fill" {
                        assert!(item.get("w").is_none(), "a fill has no stroke width");
                    } else {
                        let w = item["w"].as_f64().expect("stroke width");
                        assert!((0.0..=f64::from(MAX_STROKE)).contains(&w), "{w}");
                        if let Some(r) = item.get("r") {
                            assert!(r.is_boolean());
                        }
                    }
                }
                "text" | "icon" => {
                    let size = item["size"].as_f64().expect("size");
                    assert!((4.0..=120.0).contains(&size), "{size}");
                    for axis in ["x", "y"] {
                        assert!(item[axis].as_f64().expect(axis).is_finite());
                    }
                    if k == "text" {
                        let s = item["s"].as_str().expect("the string");
                        assert!(!s.is_empty() && s.chars().count() <= MAX_TEXT, "{s}");
                    } else {
                        let n = item["n"].as_str().expect("the icon name");
                        assert!(!n.is_empty() && !n.starts_with("mdi-"), "{n} should be normalised");
                    }
                }
                other => panic!("unknown item kind {other}"),
            }
        }
    }

    fn texts(items: &[Value]) -> Vec<String> {
        items
            .iter()
            .filter(|i| i["k"] == "text")
            .map(|i| i["s"].as_str().unwrap_or_default().to_string())
            .collect()
    }

    /// The biggest text on the screen: the number a value mode is showing.
    fn biggest(items: &[Value]) -> Value {
        items
            .iter()
            .filter(|i| i["k"] == "text")
            .max_by_key(|i| i["size"].as_f64().expect("size") as i64)
            .expect("some text")
            .clone()
    }

    #[test]
    fn every_position_draws_a_well_formed_scene() {
        let scratch = Scratch::new("every-position");
        let mut seen = [false; MODE_COUNT];
        for position in -6..40 {
            let demo = scratch.demo(position);
            seen[demo.mode()] = true;
            let items = screen_at(&demo, UNIX_EPOCH + Duration::from_secs(1_700_000_000));
            check(&items);
            assert!(
                texts(&items).iter().any(|s| s == MODE_NAMES[demo.mode()]),
                "the header names the mode: {}",
                MODE_NAMES[demo.mode()]
            );
        }
        assert!(seen.iter().all(|s| *s), "every mode was drawn");
    }

    #[test]
    fn the_header_is_the_mode_and_the_ring_lights_it() {
        for (mode, name) in MODE_NAMES.iter().enumerate() {
            let scratch = Scratch::new("ring");
            let demo = scratch.demo(mode as i64);
            let items = screen_at(&demo, UNIX_EPOCH);
            // Five ring arcs: one wide and lit, four thin and dim.
            assert_eq!(items.iter().filter(|i| i["k"] == "stroke" && i["w"] == 6.0).count(), 1);
            assert_eq!(
                items
                    .iter()
                    .filter(|i| i["k"] == "stroke" && i["w"] == 3.0 && i["c"] == DIM)
                    .count(),
                MODE_COUNT - 1,
                "the rest are dim, for mode {mode}"
            );
            assert_eq!(items[0]["s"], *name, "and the header says which mode it is");
        }
    }

    #[test]
    fn the_counter_is_the_wheel_position_and_shrinks_as_it_grows() {
        let scratch = Scratch::new("counter");
        let demo = scratch.demo(5);
        assert_eq!((demo.mode(), demo.counter()), (COUNTER, 5));
        let items = screen_at(&demo, UNIX_EPOCH);
        let number = biggest(&items);
        assert_eq!(number["s"], "5");
        assert_eq!(number["size"], 96.0);
        assert_eq!(number["w"], "bold");
        assert!(texts(&items).iter().any(|s| s == "WHEEL STEPS"));

        let demo = scratch.demo(12345);
        assert_eq!(demo.mode(), COUNTER);
        let items = screen_at(&demo, UNIX_EPOCH);
        let number = biggest(&items);
        assert_eq!(number["s"], "12345");
        assert!(
            number["size"].as_f64().expect("size") < 96.0,
            "it makes room for itself"
        );
    }

    #[test]
    fn the_dial_tracks_the_value_and_only_fills_once_it_is_off_zero() {
        let scratch = Scratch::new("dial");
        // 101 is one step past a full 0-to-100 sweep: the dial reads 0, and the mode is the dial.
        let demo = scratch.demo(101);
        assert_eq!((demo.mode(), demo.dial()), (DIAL, 0));
        let items = screen_at(&demo, UNIX_EPOCH);
        assert_eq!(
            items.iter().filter(|i| i["w"] == 14.0).count(),
            1,
            "the unfilled track, and no value arc"
        );
        assert_eq!(biggest(&items)["s"], "0");
        assert!(texts(&items).iter().any(|s| s == "100"), "the ends are labelled");

        // Six steps in: still the dial, six up the gauge.
        let demo = scratch.demo(6);
        assert_eq!((demo.mode(), demo.dial()), (DIAL, 6));
        let items = screen_at(&demo, UNIX_EPOCH);
        assert_eq!(items.iter().filter(|i| i["w"] == 14.0).count(), 2, "track and value");
        assert_eq!(biggest(&items)["s"], "6");
        assert!(texts(&items).iter().any(|s| s == "0"));
        assert!(texts(&items).iter().any(|s| s == "100"));
        // Eleven ticks, and a full gauge is as far as it goes.
        assert_eq!(
            items
                .iter()
                .filter(|i| (1.5..3.0).contains(&i["w"].as_f64().unwrap_or(0.0)))
                .count(),
            11
        );
    }

    #[test]
    fn the_typography_mode_shows_weights_anchors_and_a_tracking_that_moves() {
        let scratch = Scratch::new("typography");
        let mut seen = Vec::new();
        // Every fifth position is the text mode; the tracking cycles every third one.
        for position in [2, 7, 12, 17] {
            let demo = scratch.demo(position);
            assert_eq!(demo.mode(), TEXT);
            let items = screen_at(&demo, UNIX_EPOCH);
            let tracking = items.iter().find(|i| i["s"] == "TRACKING").expect("the tracking line");
            assert_eq!(tracking["a"], "start");
            seen.push(tracking.get("t").and_then(Value::as_f64).unwrap_or(0.0));
            assert!(texts(&items).iter().any(|s| s == "Regular 13 px"));
            assert!(items.iter().any(|i| i["s"] == "Bold 13 px" && i["w"] == "bold"));
            // The three anchors, each with its guide: middle is the default and stays off the wire.
            assert_eq!(items.iter().find(|i| i["s"] == "start").expect("start")["a"], "start");
            assert!(
                items
                    .iter()
                    .find(|i| i["s"] == "middle")
                    .expect("middle")
                    .get("a")
                    .is_none()
            );
            assert_eq!(items.iter().find(|i| i["s"] == "end").expect("end")["a"], "end");
            assert_eq!(items.iter().filter(|i| i["k"] == "stroke" && i["w"] == 1.0).count(), 3);
            // And one quadratic curve: `Q` is in the path data.
            assert!(
                items
                    .iter()
                    .any(|i| i["k"] == "stroke" && i["d"].as_str().is_some_and(|d| d.contains('Q'))),
                "the curve is drawn with Q"
            );
        }
        assert_eq!(seen, [3.0, 1.5, 0.0, 3.0], "the wheel moves the tracking");
    }

    #[test]
    fn the_icon_grid_uses_names_the_device_has_and_picks_one() {
        let scratch = Scratch::new("icons");
        let mut picked = Vec::new();
        // Every fifth position is the icon grid, and eight of them walk the whole grid.
        for i in 0..8 {
            let demo = scratch.demo(3 + 5 * i);
            assert_eq!(demo.mode(), ICONS);
            let items = screen_at(&demo, UNIX_EPOCH);
            let icons: Vec<String> = items
                .iter()
                .filter(|i| i["k"] == "icon")
                .map(|i| i["n"].as_str().unwrap_or_default().to_string())
                .collect();
            assert_eq!(icons.len(), ICON_GRID.len());
            for name in ICON_GRID {
                assert!(icons.contains(&name.to_string()), "{name} is missing");
            }
            // Exactly one cell is lit: a highlight behind it, and its own ink in teal — plus the
            // header and the lit ring segment, four teal items in all.
            assert_eq!(items.iter().filter(|i| i["k"] == "fill").count(), 1);
            assert_eq!(
                items.iter().filter(|i| i["c"] == TEAL).count(),
                4,
                "header, ring, cell, label"
            );
            picked.push(demo.pick(8));
        }
        picked.sort_unstable();
        assert_eq!(picked, [0, 1, 2, 3, 4, 5, 6, 7], "the wheel walks the whole grid");
    }

    #[test]
    fn the_data_mode_waits_politely_and_then_shows_the_payload() {
        let scratch = Scratch::new("data");
        let demo = scratch.demo(4);
        assert_eq!(demo.mode(), DATA);
        let items = screen_at(&demo, UNIX_EPOCH);
        assert!(texts(&items).iter().any(|s| s == "NO DATA YET"));
        assert!(items.iter().any(|i| i["k"] == "icon" && i["n"] == "eye"));
        check(&items);

        let at = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let mut demo = scratch.demo(4);
        demo.arrived_at(
            &json!({"stop": "Gare Centrale", "departures": [{"route": "55", "minutes": 3}]}),
            at,
        );
        let items = screen_at(&demo, at + Duration::from_secs(9));
        let header = items
            .iter()
            .find(|i| i["k"] == "text" && i["s"].as_str().is_some_and(|s| s.contains("BYTES")))
            .expect("the payload line");
        let header = header["s"].as_str().expect("a string");
        assert!(header.contains("22:13:20Z"), "{header}");
        assert!(header.contains("9S AGO"), "{header}");
        assert!(texts(&items).iter().any(|s| s.contains("Gare Centrale")));
        assert!(!texts(&items).iter().any(|s| s == "NO DATA YET"));
        check(&items);

        // A payload too long for one page pages, and says which page it is on.
        let mut demo = scratch.demo(9);
        assert_eq!(demo.mode(), DATA);
        demo.arrived_at(
            &json!({"rows": (0..80).map(|i| json!({"i": i})).collect::<Vec<_>>()}),
            at,
        );
        let items = screen_at(&demo, at);
        assert!(
            texts(&items).iter().any(|s| s == "10/10"),
            "page {} of ten: {:?}",
            demo.pick(10) + 1,
            texts(&items)
        );
        check(&items);
    }

    #[test]
    fn the_event_and_the_screen_agree_on_the_mode() {
        let scratch = Scratch::new("event");
        let mut demo = Demo::load_from(Store::at(&scratch.0));
        for _ in 1..=12 {
            demo.wheel(1);
            assert_eq!(demo.event()["name"], MODE_NAMES[demo.mode()]);
            let items = screen_at(&demo, UNIX_EPOCH);
            assert_eq!(items[0]["s"], MODE_NAMES[demo.mode()]);
        }
    }
}
