// Right column: what the selected plugin's MANIFEST declares (permissions,
// routes, slot contributions) plus the dispatch event log — the playground's
// x-ray of the sandbox boundary.
import type { PluginManifest } from '../../playground/registry';
import type { LoadedPlugin } from './plugins';

export type EventLogEntry = {
	id: number;
	action: string;
	targetPluginId: string;
	message: string;
	originContributionId?: string;
};

export function DetailsPanel({
	selected,
	activeRoutePattern,
	events,
	onNavigate,
}: {
	selected: LoadedPlugin | undefined;
	activeRoutePattern: string | undefined;
	events: EventLogEntry[];
	onNavigate: (to: string) => void;
}) {
	return (
		<aside className="pg-details" aria-label="Plugin details">
			<section>
				<h2>{selected?.envelope.manifest.name ?? 'Loading'}</h2>
				<p>{selected?.asset.sourcePath}</p>
				<div className="pg-tags">
					{(selected?.asset.tags ?? []).map((tag) => <span key={tag}>{tag}</span>)}
				</div>
			</section>
			<section>
				<h2>Permissions</h2>
				<div className="pg-permission-list">
					{((selected?.envelope.manifest as PluginManifest | undefined)?.permissions ?? []).map((permission) => (
						<span key={permission.permission}>{permission.permission}</span>
					))}
				</div>
			</section>
			<section>
				<h2>Routes</h2>
				<div className="pg-route-list">
					{(selected?.envelope.manifest.routes ?? []).map((route) => (
						<button
							key={route.pattern}
							type="button"
							className={route.pattern === activeRoutePattern ? 'is-active' : ''}
							disabled={route.pattern.includes(':')}
							title={route.screenKind}
							onClick={() => onNavigate(route.pattern)}
						>
							{route.pattern}
						</button>
					))}
					{!selected?.envelope.manifest.routes?.length && (
						<p>No declared routes; using {selected?.asset.routePattern ?? 'registry pattern'}.</p>
					)}
				</div>
			</section>
			<section>
				<h2>Slot Contributions</h2>
				<div className="pg-permission-list">
					{(selected?.envelope.manifest.slotContributions ?? []).map((contribution) => (
						<span key={contribution.id}>{contribution.target}</span>
					))}
					{!selected?.envelope.manifest.slotContributions?.length && <p>No slot contributions.</p>}
				</div>
			</section>
			<section>
				<h2>Event Log</h2>
				<div className="pg-event-log">
					{events.length === 0 && <p>No actions yet.</p>}
					{events.map((event) => (
						<div key={event.id}>
							<strong>{event.action}</strong>
							<span>{event.message}</span>
							<small>target: {event.targetPluginId}</small>
							{event.originContributionId && <small>contribution: {event.originContributionId}</small>}
						</div>
					))}
				</div>
			</section>
		</aside>
	);
}
