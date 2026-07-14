import type { NavigationContext, NavigationItem } from '../api/contracts';

type LazyText = string | (() => string);

type RegisteredNavigationItem = Omit<NavigationItem<any>, 'label' | 'shortLabel'> & {
  pluginId: string;
  label: LazyText;
  shortLabel?: LazyText;
};

export type ResolvedNavigationItem = Omit<NavigationItem<any>, 'label' | 'shortLabel'> & {
  pluginId: string;
  label: string;
  shortLabel?: string;
};

export class NavigationRegistry {
  private items: RegisteredNavigationItem[] = [];

  register(item: Omit<NavigationItem<any>, 'label' | 'shortLabel'> & { label: LazyText; shortLabel?: LazyText }, pluginId: string) {
    this.items.push({ ...item, pluginId });
  }

  async getAvailable(ctx: NavigationContext): Promise<ResolvedNavigationItem[]> {
    const list: ResolvedNavigationItem[] = [];
    for (const item of this.items) {
      const allowed = item.when ? await item.when(ctx) : true;
      if (allowed) {
        list.push({
          ...item,
          label: typeof item.label === 'function' ? item.label() : item.label,
          shortLabel:
            typeof item.shortLabel === 'function'
              ? item.shortLabel()
              : item.shortLabel
        });
      }
    }
    list.sort((a, b) => (b.priority ?? 0) - (a.priority ?? 0));
    return list;
  }
}
