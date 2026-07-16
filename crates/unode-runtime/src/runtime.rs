use unode::core::permissions::PermissionGuard;
use unode::core::slot::SlotRegistry;

use crate::host::RuntimeSandbox;
use crate::registry::{ActionRegistry, CommandRegistry, NavigationRegistry, RouteRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTarget {
    Web,
    Tui,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeFeatures {
    pub shell_navigation: bool,
    pub command_palette: bool,
    pub route_tabs_chrome: bool,
    pub retained_screen_patches: bool,
}

impl RuntimeFeatures {
    pub const fn web_defaults() -> Self {
        Self {
            shell_navigation: true,
            command_palette: true,
            route_tabs_chrome: true,
            retained_screen_patches: true,
        }
    }

    pub const fn tui_defaults() -> Self {
        Self {
            shell_navigation: true,
            command_palette: true,
            route_tabs_chrome: true,
            retained_screen_patches: true,
        }
    }
}

pub struct HostedRuntime<Ctx> {
    pub target: RuntimeTarget,
    pub features: RuntimeFeatures,
    pub sandbox: RuntimeSandbox,
    pub routes: RouteRegistry,
    pub navigation: NavigationRegistry,
    pub commands: CommandRegistry<Ctx>,
    pub actions: ActionRegistry<Ctx>,
    pub slots: SlotRegistry,
}

impl<Ctx> HostedRuntime<Ctx> {
    pub fn new(target: RuntimeTarget, guard: PermissionGuard) -> Self {
        let features = match target {
            RuntimeTarget::Web => RuntimeFeatures::web_defaults(),
            RuntimeTarget::Tui => RuntimeFeatures::tui_defaults(),
        };

        Self {
            target,
            features,
            sandbox: RuntimeSandbox::new(target, guard),
            routes: RouteRegistry::default(),
            navigation: NavigationRegistry::default(),
            commands: CommandRegistry::default(),
            actions: ActionRegistry::default(),
            slots: SlotRegistry::default(),
        }
    }
}
