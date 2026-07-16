// The playground's host runtime bootstrap — the same role `runtime.ts` plays
// in `examples/web-*`: instantiate the wasm host core (normalization,
// reactivity tracking, patch planning) and expose the session the app drives.
import { HostSession } from 'unode-web-core';

import * as webHostModule from '../../playground/pkg/unode_web_host.js';
import webHostWasmUrl from '../../playground/pkg/unode_web_host_bg.wasm?url';

export const LOCALE = 'en';

export function createHostSession(): Promise<HostSession> {
	return HostSession.create(webHostModule as never, webHostWasmUrl, LOCALE);
}
