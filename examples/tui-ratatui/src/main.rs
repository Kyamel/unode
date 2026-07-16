//! Minimal ratatui host — the TUI counterpart of `examples/web-react` and
//! `examples/web-svelte`. Loads the counter plugin WASM and renders it
//! through the renderer SDK; `app.rs` is the App, `button.rs` the host's
//! native Button backing `action` nodes.
//!
//! Keys: arrows move focus, Enter dispatches the focused action, `q` quits.

use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

mod app;
mod button;

use app::App;

fn main() -> Result<()> {
    let wasm_path = find_counter_wasm().context(
        "counter plugin not built; run `cargo build --manifest-path plugins/counter/Cargo.toml --target wasm32-unknown-unknown`",
    )?;
    let mut app = App::new(&wasm_path)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let result = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn find_counter_wasm() -> Option<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    ["debug", "release"]
        .iter()
        .map(|profile| {
            root.join("plugins/counter/target/wasm32-unknown-unknown")
                .join(profile)
                .join("web_counter_plugin.wasm")
        })
        .filter(|path| path.exists())
        .max_by_key(|path| path.metadata().and_then(|meta| meta.modified()).ok())
}
