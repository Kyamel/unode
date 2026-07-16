// The plugin shell: host chrome (route tabs derived from the manifest) above
// the mounted plugin screen. Tabs are navigation, so clicks go through the
// app's navigate, never into the plugin.
import { UnodeScreen, type ActionRef, type ScreenStore } from 'unode-react';
import type { RouteTabsView } from 'unode-web-core';

import { PluginButton } from './Button';
import { playgroundRenderer } from './renderer';

export function PluginShell({
	store,
	tabsView,
	onAction,
	onNavigate,
}: {
	store: ScreenStore | null;
	tabsView: RouteTabsView | null;
	onAction: (action: ActionRef) => void;
	onNavigate: (to: string) => void;
}) {
	return (
		<main className="pg-main">
			{tabsView && (
				<div className="pg-tabs" role="tablist">
					{tabsView.tabs.map((tab) => (
						<button
							key={tab.to}
							type="button"
							role="tab"
							aria-selected={tab.to === tabsView.active}
							className={tab.to === tabsView.active ? 'is-active' : ''}
							onClick={() => onNavigate(tab.to)}
						>
							<span>{tab.label}</span>
							{tab.badge && <small>{tab.badge}</small>}
						</button>
					))}
				</div>
			)}
			{store ? (
				<UnodeScreen
					store={store}
					onAction={onAction}
					renderer={playgroundRenderer}
					components={{ Button: PluginButton }}
				/>
			) : (
				<p className="pg-loading">Loading plugin WASM...</p>
			)}
		</main>
	);
}
