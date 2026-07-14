use unode_runtime::{
    CommandResult, DeferredText, RegisteredCommand, RegisteredNavigationItem, RegisteredRoute,
};
use unode_tui_runtime::TuiRuntime;

pub fn register_builtin_shell(runtime: &mut TuiRuntime<()>) {
    runtime.inner.routes.register(RegisteredRoute {
        plugin_id: "org.mugens.core.home".to_string(),
        pattern: "/home".to_string(),
        screen_kind: "org.mugens.core.home".to_string(),
        priority: 100,
    });
    runtime.inner.routes.register(RegisteredRoute {
        plugin_id: "org.mugens.core.mangas.hot".to_string(),
        pattern: "/mangas/hot".to_string(),
        screen_kind: "org.mugens.core.mangas.hot".to_string(),
        priority: 100,
    });
    runtime.inner.routes.register(RegisteredRoute {
        plugin_id: "org.mugens.core.mangas.recent".to_string(),
        pattern: "/mangas/recent".to_string(),
        screen_kind: "org.mugens.core.mangas.recent".to_string(),
        priority: 100,
    });

    runtime.inner.navigation.register(RegisteredNavigationItem {
        id: "nav.home".to_string(),
        plugin_id: "org.mugens.core.home".to_string(),
        label: DeferredText::from("Home"),
        short_label: None,
        to: "/home".to_string(),
        icon: None,
        section: Some("main".to_string()),
        priority: 300,
        when: None,
    });
    runtime.inner.navigation.register(RegisteredNavigationItem {
        id: "nav.mangas.hot".to_string(),
        plugin_id: "org.mugens.core.mangas.hot".to_string(),
        label: DeferredText::from("Mangas Hot"),
        short_label: None,
        to: "/mangas/hot".to_string(),
        icon: None,
        section: Some("main".to_string()),
        priority: 200,
        when: None,
    });
    runtime.inner.navigation.register(RegisteredNavigationItem {
        id: "nav.mangas.recent".to_string(),
        plugin_id: "org.mugens.core.mangas.recent".to_string(),
        label: DeferredText::from("Mangas Recent"),
        short_label: None,
        to: "/mangas/recent".to_string(),
        icon: None,
        section: Some("main".to_string()),
        priority: 100,
        when: None,
    });

    runtime.inner.commands.register(RegisteredCommand {
        id: "goto.home".to_string(),
        plugin_id: "org.mugens.core.home".to_string(),
        title: DeferredText::from("Go to Home"),
        category: Some(DeferredText::from("Navigation")),
        keywords: vec!["home".to_string()],
        when: None,
        run: std::sync::Arc::new(|_| CommandResult::Navigate("/home".to_string())),
    });
    runtime.inner.commands.register(RegisteredCommand {
        id: "goto.mangas.hot".to_string(),
        plugin_id: "org.mugens.core.mangas.hot".to_string(),
        title: DeferredText::from("Go to Mangas Hot"),
        category: Some(DeferredText::from("Navigation")),
        keywords: vec!["mangas".to_string(), "hot".to_string()],
        when: None,
        run: std::sync::Arc::new(|_| CommandResult::Navigate("/mangas/hot".to_string())),
    });
    runtime.inner.commands.register(RegisteredCommand {
        id: "goto.mangas.recent".to_string(),
        plugin_id: "org.mugens.core.mangas.recent".to_string(),
        title: DeferredText::from("Go to Mangas Recent"),
        category: Some(DeferredText::from("Navigation")),
        keywords: vec!["mangas".to_string(), "recent".to_string()],
        when: None,
        run: std::sync::Arc::new(|_| CommandResult::Navigate("/mangas/recent".to_string())),
    });
}
