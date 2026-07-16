// Left column: the plugin catalog. Selecting a plugin is just a navigation
// to its default route — the app treats the sidebar like any other nav.
import { playgroundPluginAssets } from '../../playground/registry';

export function Sidebar({
	selectedPluginId,
	onSelect,
}: {
	selectedPluginId: string;
	onSelect: (assetId: string) => void;
}) {
	return (
		<aside className="pg-sidebar" aria-label="Playground plugins">
			<div className="pg-brand">
				<a href="/">Unode</a>
				<span>WASM Playground</span>
			</div>
			<div className="pg-plugin-list">
				{playgroundPluginAssets.map((asset) => (
					<button
						key={asset.id}
						type="button"
						className={asset.id === selectedPluginId ? 'is-selected' : ''}
						onClick={() => onSelect(asset.id)}
					>
						<strong>{asset.name}</strong>
						<span>{asset.sourcePath}</span>
					</button>
				))}
			</div>
		</aside>
	);
}
