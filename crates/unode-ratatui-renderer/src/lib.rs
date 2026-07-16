pub mod nodes;
pub mod recipes;
pub mod screen;
pub mod tui_shell;
pub mod util;

pub use recipes::{
    RatatuiBackend, TuiRecipe, TuiRenderCtx, TuiRenderer, ratatui_renderer, rect, region,
    render_vertical_children,
};
pub use screen::{
    TuiInteractiveElement, TuiInteractiveKind, TuiScreenView, collect_screen_interactions,
    render_tui_screen,
};
pub use tui_shell::{
    TuiCommandBar, TuiFocusedPane, TuiMainContent, TuiMainPanel, TuiNavItem, TuiShellView,
    render_tui_shell,
};
pub use unode_renderer::{
    Backend, FocusCursor, NodeKind, Recipe, Region, RenderCtx, Renderer, RendererBuilder,
    define_renderer,
};
