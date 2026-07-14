import type { SectionContribution, ScreenContributionContext } from '../api/contracts';
import type { HostApi } from '../api/host';
import type { CanonicalUiNode } from '../core/normalize';
import { MemoryStateStore } from '../core/state';
import { normalizeNode } from '../core/normalize';
import type {
	PluginI18nApi,
	PluginManifest,
	ResolvedRoute,
	SlotContribution
} from '../core/runtime';
import { createRenderContext } from '../runtime/context';

export type RegisteredSection = SectionContribution<any> & { pluginId: string };

export class ScreenRegistry {
	private sections: RegisteredSection[] = [];

	registerSection(def: SectionContribution<any>, pluginId: string) {
		this.sections.push({ ...def, pluginId });
	}

	registerCoreSection<THostApi extends HostApi>(
		def: SlotContribution<THostApi>,
		plugin: PluginManifest,
		host: THostApi,
		i18n: PluginI18nApi
	) {
		const when = def.when;
		this.registerSection(
			{
				id: def.id,
				target: def.target,
				priority: def.priority,
				when: when
					? async (ctx) => {
							const route: ResolvedRoute = {
								pattern: ctx.screenKind,
								params: ctx.routeParams,
								query: Object.fromEntries(ctx.query.entries())
							};
							return await when!(
								createRenderContext(plugin, host, route, new MemoryStateStore(), i18n)
							);
						}
					: undefined,
				render: async (ctx) => {
					const route: ResolvedRoute = {
						pattern: ctx.screenKind,
						params: ctx.routeParams,
						query: Object.fromEntries(ctx.query.entries())
					};
					return normalizeNode(
						await def.render(createRenderContext(plugin, host, route, new MemoryStateStore(), i18n))
					);
				}
			},
			plugin.id
		);
	}

	async resolveSections(ctx: ScreenContributionContext, availableSlots: string[]) {
		const active: Array<{
			target: string;
			node: CanonicalUiNode;
			priority: number;
			pluginId: string;
		}> = [];

		for (const section of this.sections) {
			if (!availableSlots.includes(section.target)) continue;
			const allowed = section.when ? await section.when(ctx) : true;
			if (!allowed) continue;
			const node = await section.render(ctx);
			active.push({
				target: section.target,
				node,
				priority: section.priority ?? 0,
				pluginId: section.pluginId
			});
		}

		active.sort((a, b) => b.priority - a.priority);
		return active;
	}
}
