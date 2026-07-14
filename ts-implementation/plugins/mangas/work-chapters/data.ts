import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { ChapterSummary } from '$lib/plugins-bridge/models';

export type WorkChaptersResult = {
	chapters: ChapterSummary[];
	error: string | null;
};

function toErrorMessage(error: unknown): string {
	if (error instanceof Error && error.message.trim()) return error.message;
	if (typeof error === 'string' && error.trim()) return error;
	return 'Unknown error.';
}

export async function loadWorkChapters(
	api: Pick<MugenHostApi, 'catalog'>,
	workId: string
): Promise<WorkChaptersResult> {
	if (!workId) return { chapters: [], error: 'Missing work id.' };

	try {
		const chapters = await api.catalog.listChaptersByWork(workId);
		return { chapters, error: null };
	} catch (error: unknown) {
		return { chapters: [], error: toErrorMessage(error) };
	}
}
