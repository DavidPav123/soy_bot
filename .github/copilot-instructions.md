# AI Coding Agent Guide: soy_bot + rust-sc2

## Overview
- Project: `soy_bot` app crate using a vendored `rust-sc2` library to build StarCraft II bots.
- Layout: root `src/main.rs` (your bot) + `rust-sc2/` (library), `rust-sc2/sc2-macro/` (proc macros), `rust-sc2/sc2-proto-rs/` (generated protobuf API types).
- Key docs/code: `rust-sc2/src/{lib.rs,client.rs,bot.rs,paths.rs}` and examples in `rust-sc2/examples`.

## Core Architecture
- Bot contract: implement `Player` trait callbacks — `get_player_settings`, `on_start`, `on_step`, `on_end`, `on_event`. Most are optional except `get_player_settings`.
- Macro glue: annotate your struct with `#[bot]` (from `sc2-macro`) to inject a hidden `_bot: rust_sc2::bot::Bot` and implement `Deref/DerefMut` so you can call `self.*` bot APIs directly.
- Entry points (from `rust_sc2::client`):
  - `run_vs_computer(&mut bot, Computer::new(...), map_name, LaunchOptions)`
  - `run_vs_human(&mut bot, PlayerSettings::new(...), map_name, LaunchOptions)`
  - `run_ladder_game(&mut bot, host, port, player_port, opponent_id)`
- Game loop: `client.rs` drives SC2 process launch, websocket connection, first observation, then steps until game end, invoking your `Player` methods each iteration.

## Build & Run
- Prereqs: StarCraft II installed. On Windows, path is auto-detected via `Documents/StarCraft II/ExecuteInfo.txt` or defaults to `C:/Program Files (x86)/StarCraft II`. Set `SC2PATH` to override.
- Map selection: pass the map name only (no path, no `.SC2Map`). Example: `"EternalEmpireLE"`. `client` will resolve to `.../Maps/<name>.SC2Map`.
- Run app (from repo root):
  - `cargo run`
- Run library examples (helpful references):
  - `cd rust-sc2; cargo run --example worker-rush -- local`
  - Use `--help` flags on examples for modes/args.
- Features in use: root `Cargo.toml` enables `rust-sc2` features `serde` and `rayon`. Linux headful mode uses `--features wine_sc2` (not for Windows).

## Project Conventions
- Import prelude: `use rust_sc2::prelude::*;` — brings common types: `Player`, `PlayerSettings`, `Computer`, `Race`, `Units`, `Target`, `LaunchOptions`, iterators, and ids.
- Bot struct patterns:
  - Minimal: `#[bot] #[derive(Default)] struct MyBot;` then implement `Player`.
  - Custom init: use `#[bot_new]` on a `fn new() -> Self` returning a struct literal; macro auto-fills `_bot: Default::default()`.
- Map names: always pass logical map names to `run_vs_*` (no full disk paths). See `rust-sc2/src/paths.rs`.
- Resource accounting: prefer the pattern used in docs — check `self.can_afford(UnitTypeId::X, reserve_supply)`, issue command, then `self.subtract_resources(UnitTypeId::X, reserve_supply)` to keep counters consistent.
- Placement/search: use `self.find_placement(UnitTypeId::X, pos, PlacementOptions { step, max_distance, ..Default::default() })` when building; choose an idle worker and issue `.build(...)`.
- Unit collections: use `self.units.my.{workers,structures,townhalls,...}` and iterators like `.of_type(UnitTypeId::Marine).ready().idle()`; target with `Target::Pos` or `Target::Tag`.

## Integration Details
- SC2 client launch: `client.rs` spawns the correct binary for your OS/arch, picks free ports, connects via websocket (tungstenite). `LaunchOptions` controls `realtime`, `save_replay_as`, and optional `sc2_version`.
- Paths/resolution: `SC2PATH` honored; Windows path discovered automatically. Maps directory must exist under `SC2PATH/Maps`.
- Ladder mode: ensure your app can accept ladder CLI args (`--LadderServer`, `--GamePort`, `--StartPort`, `--OpponentId`, `--RealTime`) and call `run_ladder_game`. See guidance and patterns in `rust-sc2/src/lib.rs` and examples.
- Proto layer: `sc2-proto-rs` contains generated protobufs (no high-level logic). Do not edit unless regenerating from Blizzard definitions.

## Practical Examples
- Basic bot (see `src/main.rs`):
  - `#[bot] struct WorkerRush;` -> attack with all workers in `on_start` using `self.units.my.workers` and `worker.attack(Target::Pos(self.enemy_start), false)`.
- Typical run vs computer:
  - `run_vs_computer(&mut bot, Computer::new(Race::Random, Difficulty::Medium, None), "EternalEmpireLE", Default::default())`.

## Tips for Agents
- When adding new bots/types, keep public APIs and file layout minimal; prefer extending `src/main.rs` or add new files under `src/` in the app crate, not inside `rust-sc2/`.
- If the game fails early, verify map name and `SC2PATH`. On Windows, avoid passing absolute map paths — rely on name resolution.
- Use `rayon`-enabled iterators only if feature is enabled (already on via root dependency).

If anything here seems off for your setup (e.g., map names, SC2 install path, ladder args), tell us what you’re trying to run and we’ll refine these notes.
