# Architecture

## Overview
Amp is a terminal text editor with a clear separation between state, command logic, and rendering. The main loop lives in `src/models/application/mod.rs` and ties together the workspace state (from the `scribe` crate), input events, and the TUI renderer.

## Runtime Flow
1. `Application::new` loads preferences, builds a `View`, and creates a `Workspace` based on CLI arguments.
2. `View::new` initializes the terminal and spawns an `EventListener` thread that forwards terminal events into an `mpsc` channel.
3. `Application::run` loops: render the current mode, block on the next event, then handle all queued events.
4. Key events are routed through `commands::application::handle_input`, which looks up a command based on the current mode and keymap and mutates `Application` state.

## Modules and Responsibilities
- `src/commands/`: procedural functions that mutate `Application` and its state (buffers, modes, workspace).
- `src/models/`: application state (modes, preferences, events). No direct terminal or input parsing here.
- `src/input/`: key parsing and keymap loading (including defaults in `src/input/key_map/default.yml`).
- `src/view/`: terminal rendering, scroll regions, render caches, theming, and event listening.
- `src/presenters/`: mode-specific renderers that format workspace state into view components.
- `src/util/`: shared helpers (tokens, lexing, selection helpers).
- `src/themes/`: bundled TextMate theme files.

## Modes and Presentation
Modes are defined in `src/models/application/modes/` and collected into the `Mode` enum. Each mode has a corresponding presenter under `src/presenters/modes/` that renders the UI for that mode using a `Presenter` built from the `View`.

## Command Registry and Keymaps
Commands are public functions in `src/commands/` that accept `&mut Application`. `build.rs` scans these modules and generates a command registry mapping string names like `buffer::save` to function pointers. Keymaps map mode-specific keys to those command names:
- Default bindings: `src/input/key_map/default.yml`.
- User overrides: preferences YAML (see `src/models/application/preferences/default.yml` for defaults).

This registry enables command mode and dynamic keymap configuration.

## Rendering Pipeline
`View` owns the terminal and a set of `ScrollableRegion`/`RenderCache` structures per buffer. `Presenter` builds a `TerminalBuffer` and writes styled cells before flushing to the terminal. Syntax highlighting is driven by `syntect` and the `scribe` workspace syntax set.

## Configuration Model
Preferences are YAML-backed and loaded from a platform-specific config directory at runtime. They control themes, soft tabs, syntax associations, keymaps, and other UI behaviors. See the docs in `documentation/pages/configuration.md` for user-facing details.
