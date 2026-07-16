//! Unode TUI Playground — the terminal twin of the web playground. Each
//! concern lives in its own module:
//!
//! - `plugin_registry` — discovers and loads every plugin wasm under
//!   `plugins/`, registering their manifest routes
//! - `shell_registry`  — the shell's own surface (the Home screen)
//! - `renderer`        — the engine: how semantic nodes are painted
//! - `app`             — shell state and the render/dispatch loop
//! - `route`           — pathname/query parsing

use anyhow::Result;

mod app;
mod plugin_registry;
mod renderer;
mod route;
mod shell_registry;
#[cfg(test)]
mod tests;

use app::App;

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()
}
