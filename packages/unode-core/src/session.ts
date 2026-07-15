// Thin typed wrapper over the `unode-web-host` wasm-bindgen module.
//
// The wasm-bindgen `WebSession` exposes JSON-in/JSON-out methods; this wrapper
// parses them into the typed IR the renderer consumes.

import type { IrPatchOp, IrScreen } from "unode-renderer";

/** Shape of the generated wasm-bindgen module (subset we use). */
export interface WebHostModule {
  default: (input?: unknown) => Promise<unknown>;
  WebSession: new (locale: string) => WasmWebSession;
}

interface WasmWebSession {
  setRoute(routeJson: string): void;
  mount(screenJson: string, seedJson: string): string;
  initialPatches(): string;
  applyWrites(writesJson: string): string;
  stateSnapshot(): string;
  free(): void;
}

export interface ResolvedRoute {
  pattern: string;
  params?: Record<string, string>;
  query?: Record<string, string>;
}

export class HostSession {
  private constructor(private readonly inner: WasmWebSession) {}

  /**
   * Initialize the wasm-bindgen module and create a session.
   * `mod` is the imported generated module; `wasmUrl` is the `.wasm` asset URL.
   */
  static async create(
    mod: WebHostModule,
    wasmUrl: string,
    locale: string,
  ): Promise<HostSession> {
    await mod.default(wasmUrl);
    return new HostSession(new mod.WebSession(locale));
  }

  setRoute(route: ResolvedRoute): void {
    this.inner.setRoute(JSON.stringify(route));
  }

  /** Normalize + track a rendered screen; returns the IR to mount. */
  mount(screen: unknown, seed: Record<string, unknown> = {}): IrScreen {
    return JSON.parse(this.inner.mount(JSON.stringify(screen), JSON.stringify(seed))) as IrScreen;
  }

  /** Patch ops that resolve every binding against seeded state. */
  initialPatches(): IrPatchOp[] {
    return JSON.parse(this.inner.initialPatches()) as IrPatchOp[];
  }

  /** Apply state writes; returns the patch ops to re-apply. */
  applyWrites(writes: Record<string, unknown>): IrPatchOp[] {
    return JSON.parse(this.inner.applyWrites(JSON.stringify(writes))) as IrPatchOp[];
  }

  stateSnapshot(): Record<string, unknown> {
    return JSON.parse(this.inner.stateSnapshot());
  }

  dispose(): void {
    this.inner.free();
  }
}
