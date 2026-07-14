import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkDetails } from '$lib/plugins-bridge/models';

export type WorkMetaResult = {
	work: WorkDetails | null;
	error: string | null;
};

function toErrorMessage(error: unknown): string {
	if (error instanceof Error && error.message.trim()) return error.message;
	if (typeof error === 'string' && error.trim()) return error;
	return 'Unknown error.';
}

export async function loadWorkMeta(
	api: MugenHostApi,
	pluginId: string,
	workId: string
): Promise<WorkMetaResult> {
	if (!workId) {
		return { work: null, error: 'Missing work id.' };
	}

	const cacheKey = `work:${workId}`;

	try {
		const cachedWork = await api.storage.getScoped<WorkDetails>(pluginId, cacheKey);
		if (cachedWork) {
			return { work: cachedWork, error: null };
		}

		const work = await api.catalog.getWorkById(workId);
		await api.storage.setScoped(pluginId, cacheKey, work);
		return { work, error: null };
	} catch (error: unknown) {
		return {
			work: null,
			error: toErrorMessage(error)
		};
	}
}
