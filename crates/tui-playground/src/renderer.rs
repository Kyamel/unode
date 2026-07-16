//! The renderer declaration — where THIS host decides how semantic nodes
//! look, mirroring the web playground's `recipes.ts` + `renderer.ts`.
//!
//! `ratatui_renderer()` seeds a builder with the default recipe per node
//! kind. Customization has three layers, cheapest first:
//!
//! ```ignore
//! ratatui_renderer()
//!     // 1. restyle only the painting; the registered measure is kept
//!     .override_render(NodeKind::Badge, |ctx, node, area| { /* paint */ })
//!     // 2. decorate the default instead of replacing it
//!     .wrap(NodeKind::Action, |inner, ctx, node, area| {
//!         /* adornments */ inner(ctx, node, area);
//!     })
//!     // 3. full typed recipe (measure + render)
//!     .recipes([TuiRecipe::text(measure, render)])
//!     .build()
//! ```
//!
//! The playground intentionally ships the defaults so every plugin screen is
//! shown exactly as a bare host would render it.

use unode_ratatui_renderer::{TuiRenderer, ratatui_renderer};

pub(crate) fn playground_renderer() -> TuiRenderer {
    ratatui_renderer().build()
}
