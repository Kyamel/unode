import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkSummary } from '$lib/plugins-bridge/models';

export type BrowseWorksResult = {
  works: WorkSummary[];
  error: string | null;
};

function toErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim()) return error.message;
  if (typeof error === 'string' && error.trim()) return error;
  return 'Unknown error.';
}

export async function loadBrowseWorks(
  host: MugenHostApi,
  input?: { limit?: number }
): Promise<BrowseWorksResult> {
  try {
    const page = await host.catalog.listWorks({ limit: input?.limit ?? 48 });
    return { works: page.data ?? [], error: null };
  } catch (err: unknown) {
    return { works: [], error: toErrorMessage(err) };
  }
}
