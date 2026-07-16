use unode_runtime::{
    CommandResult, DeferredText, RegisteredCommand, RegisteredNavigationItem, RegisteredRoute,
};
use unode_tui_runtime::TuiRuntime;

/// Registers the shell's own surface: a single Home landing screen. Every
/// other sidebar entry comes from the discovered plugins' manifests.
pub fn register_builtin_shell(runtime: &mut TuiRuntime<()>) {
    runtime.inner.routes.register(RegisteredRoute {
        plugin_id: "dev.unode.shell.home".to_string(),
        pattern: "/home".to_string(),
        screen_kind: "dev.unode.shell.home".to_string(),
        priority: 100,
    });

    runtime.inner.navigation.register(RegisteredNavigationItem {
        id: "nav.home".to_string(),
        plugin_id: "dev.unode.shell.home".to_string(),
        label: DeferredText::from("Home"),
        short_label: None,
        to: "/home".to_string(),
        icon: None,
        section: Some("main".to_string()),
        priority: 500,
        when: None,
    });

    runtime.inner.commands.register(RegisteredCommand {
        id: "goto.home".to_string(),
        plugin_id: "dev.unode.shell.home".to_string(),
        title: DeferredText::from("Go to Home"),
        category: Some(DeferredText::from("Navigation")),
        keywords: vec!["home".to_string()],
        when: None,
        run: std::sync::Arc::new(|_| CommandResult::Navigate("/home".to_string())),
    });
}
