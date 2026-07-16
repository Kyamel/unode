// Native instantiation of a plugin `.wasm` (raw C ABI, `wasm32-unknown-unknown`).
//
// The plugin lives in isolated linear memory. The only way in or out is this
// ABI: JSON in via `unode_alloc` + an export call, JSON out via a paired
// `_result_len` export. That isolation is the sandbox.

interface PluginExports {
  memory: WebAssembly.Memory;
  unode_alloc(len: number): number;
  unode_dealloc(ptr: number, len: number): void;
  plugin_manifest(): number;
  plugin_manifest_len(): number;
  plugin_load(ptr: number, len: number): number;
  plugin_load_result_len(): number;
  plugin_render(ptr: number, len: number): number;
  plugin_render_result_len(): number;
  plugin_render_slot(ptr: number, len: number): number;
  plugin_render_slot_result_len(): number;
  plugin_dispatch(ptr: number, len: number): number;
  plugin_dispatch_result_len(): number;
}

/** Optional host-call handler: operation name + params -> JSON response. */
export type HostCallHandler = (
  operation: string,
  params: Record<string, unknown>,
) => unknown;

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

export class PluginInstance {
  private ex!: PluginExports;
  private lastHostResultLen = 0;

  private constructor(private readonly onHostCall?: HostCallHandler) {}

  static async instantiate(
    wasm: BufferSource | Response | PromiseLike<Response>,
    onHostCall?: HostCallHandler,
  ): Promise<PluginInstance> {
    const self = new PluginInstance(onHostCall);
    const imports: WebAssembly.Imports = {
      unode: {
        host_call: (ptr: number, len: number) => self.handleHostCall(ptr, len),
        host_call_result_len: () => self.lastHostResultLen,
      },
    };

    const source =
      wasm instanceof Response || (wasm as PromiseLike<Response>)?.then
        ? await WebAssembly.instantiateStreaming(wasm as Response, imports)
        : await WebAssembly.instantiate(wasm as BufferSource, imports);

    self.ex = source.instance.exports as unknown as PluginExports;
    return self;
  }

  private bytes(): Uint8Array {
    // Re-view every access: the buffer detaches when memory grows.
    return new Uint8Array(this.ex.memory.buffer);
  }

  private writeJson(value: unknown): { ptr: number; len: number } {
    const encoded = textEncoder.encode(JSON.stringify(value));
    const ptr = this.ex.unode_alloc(encoded.length);
    this.bytes().set(encoded, ptr);
    return { ptr, len: encoded.length };
  }

  private readJson<T>(ptr: number, len: number): T {
    const slice = this.bytes().subarray(ptr, ptr + len);
    return JSON.parse(textDecoder.decode(slice)) as T;
  }

  private call<T>(
    fn: (ptr: number, len: number) => number,
    lenFn: () => number,
    request: unknown,
  ): T {
    const { ptr, len } = this.writeJson(request);
    const resultPtr = fn.call(this.ex, ptr, len);
    return this.readJson<T>(resultPtr, lenFn.call(this.ex));
  }

  private handleHostCall(ptr: number, len: number): number {
    const request = this.readJson<{ operation: string; params?: Record<string, unknown> }>(
      ptr,
      len,
    );
    const response = this.onHostCall?.(request.operation, request.params ?? {}) ?? {
      ok: false,
      error: `unhandled host call: ${request.operation}`,
    };
    const encoded = textEncoder.encode(JSON.stringify(response));
    const responsePtr = this.ex.unode_alloc(encoded.length);
    this.bytes().set(encoded, responsePtr);
    this.lastHostResultLen = encoded.length;
    return responsePtr;
  }

  manifest<T = unknown>(): T {
    const ptr = this.ex.plugin_manifest();
    return this.readJson<T>(ptr, this.ex.plugin_manifest_len());
  }

  load<T = unknown>(request: unknown): T {
    return this.call<T>(this.ex.plugin_load, this.ex.plugin_load_result_len, request);
  }

  render<T = unknown>(request: unknown): T {
    return this.call<T>(this.ex.plugin_render, this.ex.plugin_render_result_len, request);
  }

  renderSlot<T = unknown>(request: unknown): T {
    return this.call<T>(
      this.ex.plugin_render_slot,
      this.ex.plugin_render_slot_result_len,
      request,
    );
  }

  dispatch<T = unknown>(request: unknown): T {
    return this.call<T>(this.ex.plugin_dispatch, this.ex.plugin_dispatch_result_len, request);
  }
}
