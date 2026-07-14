/**
 * unode/transport.ts
 * ─────────────────────────────────────────────────────────────────────────────
 * JSON transport layer for the unode AST.
 *
 * Converts between CanonicalScreen (in-process TypeScript objects) and a
 * wire-safe JSON envelope. Used when screens need to travel across a process
 * boundary: plugin Worker → host, Deno TUI ← remote server, etc.
 *
 * The envelope adds:
 *   - Schema version for forward compatibility
 *   - Timestamp for cache invalidation
 *   - Optional screenKind for routing on the receiving end
 *
 * Usage — sending:
 *   const json = screenToJson(canonicalScreen, { screenKind: 'catalog.browse' });
 *   await fetch('/screens/browse', { method: 'POST', body: json });
 *
 * Usage — receiving:
 *   const result = screenFromJson(responseText);
 *   if (!result.ok) throw new Error(result.error);
 *   renderer.mount(result.screen);
 *
 * Usage — inspecting in the browser console or logs:
 *   console.log(screenToJson(screen, { pretty: true }));
 * ─────────────────────────────────────────────────────────────────────────────
 */

import { UNODE_AST_VERSION } from './ast';
import type { CanonicalScreen, TransportScreen } from './normalize';
import { normalizeScreen } from './normalize';
import type { ScreenNode } from './ast';

// ─────────────────────────────────────────────────────────────────────────────
// §1  Wire envelope
// ─────────────────────────────────────────────────────────────────────────────

/**
 * The JSON envelope that wraps a CanonicalScreen for transport.
 * Every field is JSON-safe by construction.
 */
export interface ScreenEnvelope {
  /** Always "unode-screen". Lets receivers identify the payload type. */
  readonly type: 'unode-screen';

  /** AST schema version. Receivers should reject incompatible versions. */
  readonly v: string;

  /** ISO 8601 timestamp of when the envelope was created. */
  readonly ts: string;

  /** Optional screen kind for routing on the receiving end. */
  readonly screenKind?: string;

  /** The canonical screen — fully serializable, no functions or class instances. */
  readonly screen: TransportScreen;
}

/**
 * Options for screenToJson().
 */
export interface SerializeOptions {
  /** Screen kind to embed in the envelope for routing. */
  screenKind?: string;

  /** Pretty-print the JSON output. Useful for debugging and logs. Default: false. */
  pretty?: boolean;
}

/**
 * Result of screenFromJson() — discriminated union so callers are forced
 * to handle both the success and error cases.
 */
export type DeserializeResult =
  | { readonly ok: true; readonly screen: TransportScreen; readonly envelope: ScreenEnvelope }
  | { readonly ok: false; readonly error: string };

// ─────────────────────────────────────────────────────────────────────────────
// §2  Serialization
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Serialize a CanonicalScreen to a JSON string ready for transport.
 *
 * The screen must already be normalized (output of normalizeScreen()).
 * If you have a raw ScreenNode, use screenNodeToJson() instead.
 *
 * The result is deterministic for the same input — no random fields,
 * no unstable ordering. Safe to use as a cache key or for diffing.
 */
export function screenToJson(
  screen: TransportScreen,
  options?: SerializeOptions
): string {
  const envelope: ScreenEnvelope = {
    type: 'unode-screen',
    v: UNODE_AST_VERSION,
    ts: new Date().toISOString(),
    screenKind: options?.screenKind,
    screen,
  };

  return options?.pretty
    ? JSON.stringify(envelope, null, 2)
    : JSON.stringify(envelope);
}

/**
 * Convenience: normalize a raw ScreenNode and serialize to JSON in one step.
 */
export function screenNodeToJson(
  node: ScreenNode,
  options?: SerializeOptions
): string {
  return screenToJson(normalizeScreen(node), options);
}

// ─────────────────────────────────────────────────────────────────────────────
// §3  Deserialization
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Deserialize a JSON string back to a TransportScreen.
 *
 * Validates:
 *   - Valid JSON
 *   - Correct envelope type
 *   - Compatible schema version (major version must match)
 *   - Presence of a screen object
 *
 * Does NOT re-freeze the screen — the parsed object is a plain JS object.
 * If you need the frozen invariant back, pass the result through
 * normalizeScreen() again. For most renderer use cases this is unnecessary
 * since the renderer reads but never mutates the screen.
 */
export function screenFromJson(json: string): DeserializeResult {
  let parsed: unknown;

  try {
    parsed = JSON.parse(json);
  } catch (err) {
    return {
      ok: false,
      error: `Invalid JSON: ${err instanceof Error ? err.message : String(err)}`,
    };
  }

  if (!isPlainObject(parsed)) {
    return { ok: false, error: 'Expected a JSON object at the top level' };
  }

  if (parsed['type'] !== 'unode-screen') {
    return {
      ok: false,
      error: `Unknown envelope type: "${String(parsed['type'])}"`,
    };
  }

  const receivedVersion = String(parsed['v'] ?? '');
  if (!isVersionCompatible(receivedVersion, UNODE_AST_VERSION)) {
    return {
      ok: false,
      error:
        `AST version mismatch: received "${receivedVersion}", ` +
        `expected compatible with "${UNODE_AST_VERSION}". ` +
        `Major versions must match.`,
    };
  }

  if (!isPlainObject(parsed['screen'])) {
    return { ok: false, error: 'Missing or invalid "screen" field in envelope' };
  }

  if (parsed['screen']['kind'] !== 'screen') {
    return {
      ok: false,
      error: `Expected screen.kind === "screen", got "${String(parsed['screen']['kind'])}"`,
    };
  }

  return {
    ok: true,
    screen: parsed['screen'] as TransportScreen,
    envelope: parsed as ScreenEnvelope,
  };
}

// ─────────────────────────────────────────────────────────────────────────────
// §4  Utilities
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Serialize just the screen portion to a plain JS object (no envelope).
 * Useful for logging, devtools, or embedding in a larger payload.
 */
export function screenToObject(screen: TransportScreen): Record<string, unknown> {
  // JSON round-trip removes the frozen constraint and returns a plain object.
  // This is intentional — the object is for inspection, not for mounting.
  return JSON.parse(JSON.stringify(screen)) as Record<string, unknown>;
}

/**
 * Pretty-print a screen for debugging. Safe to call from browser console
 * or server logs.
 *
 * Example:
 *   import { debugScreen } from 'unode/transport';
 *   debugScreen(myCanonicalScreen); // logs to console
 */
export function debugScreen(screen: TransportScreen, label?: string): void {
  const prefix = label ? `[unode:screen:${label}]` : '[unode:screen]';
  console.group(prefix);
  console.log('version:', UNODE_AST_VERSION);
  console.log('kind:', screen.kind);
  console.log('title:', screen.title);
  console.log('_reactivity:', screen._reactivity);
  console.log('_subtreeReactivity:', screen._subtreeReactivity);
  console.log('children:', screen.children.length);
  console.log('full AST:');
  console.dir(screenToObject(screen), { depth: null });
  console.groupEnd();
}

// ─────────────────────────────────────────────────────────────────────────────
// §5  Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

/**
 * Semver-aware compatibility check.
 * Two versions are compatible if their major version numbers match.
 * "2.0.0-alpha.1" and "2.0.0" are compatible. "1.x" and "2.x" are not.
 */
function isVersionCompatible(received: string, expected: string): boolean {
  if (!received) return false;
  const receivedMajor = received.split('.')[0];
  const expectedMajor = expected.split('.')[0];
  return receivedMajor === expectedMajor;
}

/**
Para ver o JSON de uma tela no browser agora:
typescriptimport { debugScreen, screenNodeToJson } from 'unode/transport';

// Inspecionar no console
debugScreen(normalizeScreen(myScreen), 'browse');

// Ver o JSON completo
console.log(screenNodeToJson(myScreen, { pretty: true }));


Para mandar pela rede no modelo Worker → host:
typescript// Dentro do plugin Worker
const screen = plugin.routes[0].render(data, ctx);
const json = screenNodeToJson(screen, { screenKind: 'catalog.browse' });
self.postMessage({ type: 'screen', json });

// No host
const result = screenFromJson(event.data.json);
if (!result.ok) throw new Error(result.error);
renderer.mount(result.screen);
**/