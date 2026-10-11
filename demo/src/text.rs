//! Laying out a payload's JSON on a 320×320 disc.
//!
//! There is no measuring stick in the SDK — the Nest outlines the text, so an app cannot ask how
//! wide a string will be — so the demo wraps by character count, conservatively, and pages what
//! is left. The helpers here are pure and take the width as an argument, which is what makes them
//! testable without a screen.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;

/// The widest a line may be, in characters, before it is wrapped.
pub const LINE_CHARS: usize = 46;

/// How many lines are drawn on one page of the data mode.
pub const LINES_PER_PAGE: usize = 12;

/// The most lines the data mode keeps, then `...` — ten pages is more than anyone will turn.
pub const MAX_LINES: usize = 120;

/// Break `s` into lines of at most `width` characters, at spaces where it can.
///
/// A word longer than the width is broken anyway: an app cannot make the Nest draw off the canvas,
/// but it can make it draw a clipped line, and a hard break reads better. Every line keeps the
/// first line's indentation, so wrapped JSON stays readable.
pub fn wrap(s: &str, width: usize) -> Vec<String> {
    let width = width.max(8);
    let indent: String = s.chars().take_while(|c| *c == ' ').collect();
    let room = width.saturating_sub(indent.chars().count()).max(1);
    let mut lines = Vec::new();
    let mut rest = s.trim_start_matches(' ').trim_end();
    while !rest.is_empty() {
        if rest.chars().count() <= room {
            lines.push(format!("{indent}{rest}"));
            break;
        }
        let head: String = rest.chars().take(room).collect();
        let cut = match head.rfind(' ') {
            Some(i) if i > 0 => i,
            _ => head.len(),
        };
        lines.push(format!("{indent}{}", &rest[..cut]));
        rest = rest[cut..].trim_start_matches(' ');
    }
    lines
}

/// `value` pretty-printed, wrapped to `width`, and cut off after `max_lines`.
///
/// The last line is `...` when something was cut, so a clipped payload says so rather than quietly
/// ending early.
pub fn json_lines(value: &Value, width: usize, max_lines: usize) -> Vec<String> {
    let pretty = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    let mut lines: Vec<String> = Vec::new();
    let mut cut = false;
    'lines: for line in pretty.lines() {
        for wrapped in wrap(line, width) {
            if lines.len() == max_lines {
                cut = true;
                break 'lines;
            }
            lines.push(wrapped);
        }
    }
    if cut && let Some(last) = lines.last_mut() {
        *last = "...".to_string();
    }
    lines
}

/// How many pages `lines` makes, at least one.
pub fn pages(lines: usize) -> usize {
    lines.div_ceil(LINES_PER_PAGE).max(1)
}

/// One page of `lines`, and how many pages there are (for the `1/3` the reader sees).
///
/// A page number out of range wraps, so a payload that shrank under the reader's feet cannot page
/// off the end.
pub fn page(lines: &[String], page: usize) -> (Vec<&str>, usize) {
    let count = pages(lines.len());
    let start = ((page % count) * LINES_PER_PAGE).min(lines.len());
    let end = (start + LINES_PER_PAGE).min(lines.len());
    (lines[start..end].iter().map(String::as_str).collect(), count)
}

/// `14:32:07Z` — the hour, minute and second of `at`, in UTC.
///
/// The app has no timezone database in its chroot, so UTC is what it can honestly say.
pub fn clock(at: SystemTime) -> String {
    let secs = at.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs();
    format!("{:02}:{:02}:{:02}Z", (secs / 3600) % 24, (secs / 60) % 60, secs % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn wrapping_respects_the_width_and_breaks_on_spaces() {
        let lines = wrap("the quick brown fox jumps over the lazy dog", 12);
        assert!(lines.iter().all(|l| l.chars().count() <= 12), "{lines:?}");
        assert_eq!(lines.join(" "), "the quick brown fox jumps over the lazy dog");

        // A word longer than the line is broken rather than left to overflow.
        let lines = wrap("supercalifragilisticexpialidocious", 10);
        assert!(lines.iter().all(|l| l.chars().count() <= 10), "{lines:?}");
        assert_eq!(lines.concat(), "supercalifragilisticexpialidocious");

        assert_eq!(wrap("short", 40), ["short"]);
        assert_eq!(wrap("  indented", 40), ["  indented"]);
        assert!(wrap("", 40).is_empty());
        assert!(wrap("   ", 40).is_empty());

        // Whatever the width, nothing is lost and nothing overflows.
        for width in [8, 9, 17, 46] {
            for text in ["a b c d e f g h", "{\"key\": \"value\"}", "no spaces at all here"] {
                let lines = wrap(text, width);
                assert!(lines.iter().all(|l| l.chars().count() <= width.max(8)), "{lines:?}");
                assert_eq!(
                    lines.join(" ").split_whitespace().collect::<Vec<_>>(),
                    text.split_whitespace().collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn wrapping_keeps_json_indentation() {
        let text = "          {\"route\": \"55\"}";
        let lines = wrap(text, 16);
        assert!(lines.len() > 1, "{lines:?}");
        assert!(lines[0].starts_with("          {"), "{lines:?}");
        assert!(lines[1..].iter().all(|l| l.starts_with("          ")), "{lines:?}");
        assert!(lines.iter().all(|l| l.chars().count() <= 16), "{lines:?}");
    }

    #[test]
    fn wrapping_counts_characters_not_bytes() {
        // The width is clamped to 8, and characters, not bytes, count towards it.
        let lines = wrap("ééééééééééééé", 8);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines.iter().all(|l| l.chars().count() <= 8), "{lines:?}");
        assert_eq!(lines.concat(), "ééééééééééééé");
        assert_eq!(wrap("éé", 1), ["éé"], "and a width under 8 is raised to 8");
    }

    #[test]
    fn json_lines_are_pretty_wrapped_and_bounded() {
        let value = json!({"stop": "Gare Centrale", "departures": [{"route": "55", "minutes": 3}]});
        let lines = json_lines(&value, LINE_CHARS, MAX_LINES);
        assert_eq!(lines[0], "{");
        assert_eq!(lines.last().expect("a line"), "}");
        assert!(lines.iter().all(|l| l.chars().count() <= LINE_CHARS), "{lines:?}");
        assert!(lines.iter().any(|l| l.contains("\"stop\": \"Gare Centrale\"")));
        assert!(lines.iter().any(|l| l.contains("\"minutes\": 3")));

        // A payload far too big for the canvas is cut off, and says so.
        let big = json!({"big": "x".repeat(9000)});
        let lines = json_lines(&big, LINE_CHARS, MAX_LINES);
        assert_eq!(lines.len(), MAX_LINES);
        assert_eq!(lines.last().expect("a line"), "...");

        // A payload that exactly fills the budget is not marked as cut.
        let exact = json_lines(&json!(1), LINE_CHARS, 2);
        assert_eq!(exact, ["1"]);
    }

    #[test]
    fn paging_covers_every_line_once() {
        let lines: Vec<String> = (0..25).map(|i| format!("line {i}")).collect();
        assert_eq!(pages(lines.len()), 3);
        assert_eq!(pages(0), 1, "an empty page is still one page");

        let mut seen = Vec::new();
        for p in 0..pages(lines.len()) {
            let (drawn, count) = page(&lines, p);
            assert_eq!(count, 3);
            assert!(drawn.len() <= LINES_PER_PAGE);
            seen.extend(drawn.iter().map(|s| s.to_string()));
        }
        assert_eq!(seen, lines);

        // A page number out of range wraps rather than panicking.
        assert_eq!(page(&["only".to_string()], 7).0, ["only"]);
        assert_eq!(page(&lines, 3).0, page(&lines, 0).0);
        assert!(page(&[], 0).0.is_empty());
    }

    #[test]
    fn the_clock_is_utc_and_zero_padded() {
        assert_eq!(clock(UNIX_EPOCH), "00:00:00Z");
        assert_eq!(clock(UNIX_EPOCH + Duration::from_secs(1_700_000_000)), "22:13:20Z");
        assert_eq!(clock(UNIX_EPOCH + Duration::from_secs(3661)), "01:01:01Z");
        assert_eq!(
            clock(UNIX_EPOCH - Duration::from_secs(1)),
            "00:00:00Z",
            "before the epoch"
        );
    }
}
