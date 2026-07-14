import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkSummary } from '$lib/plugins-bridge/models';

export type BrowseWorksResult = {
  works: WorkSummary[];
  lastCursor: string | null;
  filterKey: string | null;
  error: string | null;
};

function toErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim()) return error.message;
  if (typeof error === 'string' && error.trim()) return error;
  return 'Unknown error.';
}

export async function loadBrowseWorksPage(
  host: MugenHostApi,
  input?: { limit?: number; cursor?: string | null }
): Promise<BrowseWorksResult> {
  try {
    const page = await host.catalog.listWorks({
      limit: input?.limit ?? 48,
      cursor: input?.cursor ?? null
    });
    const filterKey = typeof page.meta?.filterKey === 'string' ? page.meta.filterKey : null;
    return { works: page.data ?? [], lastCursor: page.lastCursor ?? null, filterKey, error: null };
  } catch (err: unknown) {
    return { works: [], lastCursor: null, filterKey: null, error: toErrorMessage(err) };
  }
}
