import type { MugenHostApi } from '$lib/plugins-bridge/host';
import type { WorkStaff } from '$lib/plugins-bridge/models';

export type WorkStaffResult = {
	staff: WorkStaff[];
	error: string | null;
};

function toErrorMessage(error: unknown): string {
	if (error instanceof Error && error.message.trim()) return error.message;
	if (typeof error === 'string' && error.trim()) return error;
	return 'Unknown error.';
}

export async function loadWorkStaff(
	api: Pick<MugenHostApi, 'catalog'>,
	workId: string
): Promise<WorkStaffResult> {
	if (!workId) return { staff: [], error: 'Missing work id.' };

	try {
		const staff = await api.catalog.listWorkStaff(workId);
		return { staff, error: null };
	} catch (error: unknown) {
		return { staff: [], error: toErrorMessage(error) };
	}
}
