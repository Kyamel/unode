import type { ActionRef as CoreActionRef, JsonValue } from '$lib/unode/core/ast';

export function navigateAction(
  to: string,
  options?: { query?: Record<string, JsonValue>; mode?: 'push' | 'replace' }
): CoreActionRef {
  return {
    type: 'unode.navigate',
    params: {
      to,
      ...(options?.mode ? { mode: options.mode } : {}),
      ...(options?.query ? { query: options.query } : {})
    }
  };
}
