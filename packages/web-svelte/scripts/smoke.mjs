// Headless proof that the *compiled wasm artifacts* drive the reactive loop —
// plugin.wasm + unode_web_host.wasm through the exact ABI pluginHost.ts uses.
// No DOM here; this validates everything up to the render layer.
//
// Run inside the nix shell, after build.sh:
//   node packages/web-svelte/scripts/smoke.mjs

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

import { initSync, WebSession } from "../pkg/unode_web_host.js";

const here = dirname(fileURLToPath(import.meta.url));
const R = (p) => resolve(here, "..", p);

const dec = new TextDecoder();
const enc = new TextEncoder();

function assert(cond, msg) {
  if (!cond) {
    console.error("FAIL:", msg);
    process.exit(1);
  }
}

// --- instantiate the core (wasm-bindgen) ---
initSync({ module: readFileSync(R("pkg/unode_web_host_bg.wasm")) });
const session = new WebSession("en");

// --- instantiate the plugin (raw C ABI), mirroring pluginHost.ts ---
let ex;
let lastLen = 0;
let pendingWrites = {};
const mem = () => new Uint8Array(ex.memory.buffer);
const writeJson = (o) => {
  const b = enc.encode(JSON.stringify(o));
  const p = ex.unode_alloc(b.length);
  mem().set(b, p);
  return [p, b.length];
};
const readJson = (p, l) => JSON.parse(dec.decode(mem().subarray(p, p + l)));
const imports = {
  unode: {
    // The sandbox boundary: buffer `state.set` calls the plugin makes.
    host_call: (p, l) => {
      const req = readJson(p, l);
      let resp = { ok: false, error: `unhandled: ${req.operation}` };
      if (req.operation === "state.set") {
        pendingWrites[String(req.params.path)] = req.params.value;
        resp = { ok: true };
      }
      const b = enc.encode(JSON.stringify(resp));
      const rp = ex.unode_alloc(b.length);
      mem().set(b, rp);
      lastLen = b.length;
      return rp;
    },
    host_call_result_len: () => lastLen,
  },
};
const { instance } = await WebAssembly.instantiate(
  readFileSync(R("demo/web_counter_plugin.wasm")),
  imports,
);
ex = instance.exports;
const pcall = (fn, lenFn, req) => {
  const [p, l] = writeJson(req);
  const rp = ex[fn](p, l);
  return readJson(rp, ex[lenFn]());
};

const route = { pattern: "/plugins/web-counter", params: {}, query: {} };

// --- the loop ---
const screen = pcall("plugin_render", "plugin_render_result_len", { route, data: {}, locale: "en" });
const ir = JSON.parse(session.mount(JSON.stringify(screen), "{}"));

// The reactive line mounts with a symbolic binding.
const findKey = (n, key) =>
  n?.p?._k === key ? n : (n?.c ?? []).map((c) => findKey(c, key)).find(Boolean);
const label = findKey(ir, "web-counter.value");
assert(label, "mounted IR contains the bound node");
assert(label.p.content?.b === "ui.countLabel", `mounts as a binding, got ${JSON.stringify(label.p.content)}`);

// Initial resolution pass turns the binding into a concrete value.
const initial = JSON.parse(session.initialPatches());
const initLabel = initial.find((op) => op.k === "web-counter.value");
assert(initLabel?.v?.v === "Count: 0", `initial resolved label is 'Count: 0', got ${JSON.stringify(initLabel?.v)}`);

function dispatchAndApply(type, expectedCount) {
  const snapshot = JSON.parse(session.stateSnapshot());
  pendingWrites = {};
  const resp = pcall("plugin_dispatch", "plugin_dispatch_result_len", {
    route,
    action: { type },
    stateSnapshot: snapshot,
    locale: "en",
  });
  assert(resp.data == null, "dispatch response carries no UI state");

  const writes = pendingWrites;
  assert(
    writes["ui.count"] === expectedCount,
    `${type} host_call set ui.count=${expectedCount}, got ${JSON.stringify(writes)}`,
  );
  const patches = JSON.parse(session.applyWrites(JSON.stringify(writes)));

  assert(patches.length === 1, `expected 1 patch, got ${patches.length}: ${JSON.stringify(patches)}`);
  assert(patches[0].o === "sp" && patches[0].k === "web-counter.value" && patches[0].f === "ct", `patch shape: ${JSON.stringify(patches[0])}`);
  assert(
    patches[0].v?.v === `Count: ${expectedCount}`,
    `patched value is 'Count: ${expectedCount}', got ${JSON.stringify(patches[0].v)}`,
  );
}

// Click sequence: dispatch → plugin host_call state.set → drain → scoped patch.
dispatchAndApply("counter.inc", 1);
dispatchAndApply("counter.inc", 2);
dispatchAndApply("counter.dec", 1);
dispatchAndApply("counter.reset", 0);

console.log("OK — real wasm artifacts: mount + inc/inc/dec/reset produced scoped patches");
