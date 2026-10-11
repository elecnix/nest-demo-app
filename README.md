# Nest demo app

A **demo app for the Nest Learning Thermostat (2nd generation) running Nest
Assistant**: it puts a card on the Nest's Devices carousel and, when you open
it, exercises every capability of the Nest app API on one screen — the ring
turns it through five modes.

- **COUNTER** — a big number that follows the ring (text items).
- **DIAL** — an arc gauge that follows the ring (fill and stroke paths, round caps).
- **TEXT** — a typography page (sizes, weights, tracking, anchors).
- **ICONS** — a grid of Material Design icons (icon items by name).
- **DATA** — the last payload Home Assistant pushed, with its size and arrival time.

Every mode change plays the standard ring tick and raises an event in Home
Assistant; the position persists in the app's own `data/`, so the app reopens
where you left it.

## Install

On the Home Assistant that talks to your Nest, open the Nest Assistant
integration → **Configure** → **Apps**: repositories tagged `nest-assistant`
appear there; install this one. The demo app's card shows up on the Nest,
after your entities. Or call the `nest_assistant.install_app` service with
`repo: elecnix/nest-demo-app`.

## What an app is

An app never touches the Nest's screen, fonts, beeper or input. It is a
statically linked ARM binary that the Nest starts when you open its card and
stops when you leave, confined to its own directory and its own user. It
speaks newline-delimited JSON on file descriptor 3: the Nest sends it `open`,
`wheel`, `data` and `stop`; it answers with scenes (fill and stroke paths,
text runs, icons — all validated and rendered by the Nest), `tick`, `event`
and `close`. Everything it may send and receive, and every limit, is
documented in the SDK's doc comments in [`sdk/`](sdk/) — `sdk/src/lib.rs` is
the entry point, `sdk/src/scene.rs` and `sdk/src/path.rs` the drawing
builders, `sdk/src/store.rs` the persistent store.

## Build and publish

```sh
./build.sh            # packages demo/dist/demo/nest-app.tar.gz for the Nest
cargo test            # the SDK and app unit tests
```

`build.sh` cross-compiles for `armv7-unknown-linux-musleabihf` (the toolchain
and target pin are in `rust-toolchain.toml`), copies the binary as `app`, and
packs exactly `manifest.json` + `app` into the gzipped tar a release must
carry. Publishing is a release with that tar as its `nest-app.tar.gz` asset,
on a repository carrying the `nest-assistant` topic — this one.

## License

MIT — see [LICENSE](LICENSE). The Material Design icons it draws by name are
rendered by the Nest itself from its own icon table.