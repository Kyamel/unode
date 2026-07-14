import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkRelation } from '$lib/plugins-bridge/models';

export type WorkRelationsResult = {
	relations: WorkRelation[];
	error: string | null;
};

function toErrorMessage(error: unknown): string {
	if (error instanceof Error && error.message.trim()) return error.message;
	if (typeof error === 'string' && error.trim()) return error;
	return 'Unknown error.';
}

export async function loadWorkRelations(
	api: Pick<MugenHostApi, 'catalog'>,
	workId: string
): Promise<WorkRelationsResult> {
	if (!workId) return { relations: [], error: 'Missing work id.' };

	try {
		const relations = await api.catalog.listWorkRelations(workId);
		return { relations, error: null };
	} catch (error: unknown) {
		return { relations: [], error: toErrorMessage(error) };
	}
}
