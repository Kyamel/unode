# Architectural Gaps

This document records the systemic runtime designs that must exist before Unode
can be treated as a general-purpose safe plugin system. These are architecture
contracts, not renderer features.

## Authority model

Security and lifecycle policy belong to the trusted host runtime:

- Web: `packages/unode-web-core`, `crates/unode-web-runtime`, and
  `crates/unode-web-host` together load plugins, route host calls, own state, and
  ask Rust to normalize/track/lower/patch.
- TUI: `crates/unode-tui-runtime` owns Wasmtime sessions, host calls, trap
  handling, and terminal lifecycle while calling `crates/unode` directly.

Renderers sit below that boundary. They draw IR, expose focus and interaction
behavior, host native components, and report symbolic actions back to the
runtime. They do not grant permissions, decide capabilities, own plugin
activation, or recover crashed plugins.

## Shared host-kernel shape

The Web and TUI implementations should converge on the same conceptual host
kernel, even when the code lives in host-specific packages:

```text
PluginInstance
  -> raw ABI module or Component Model instance

HostSession
  -> PluginInstance + PermissionProfile + StateNamespace + ResourcePolicy

CapabilityRouter
  -> validates permission + origin + namespace
  -> executes sync host calls
  -> schedules async operations

CrashSupervisor
  -> traps/timeouts/resource failures
  -> quarantine/restart/backoff/error surface

StorageProvider
  -> session/persistent plugin storage
  -> quota + versioned migrations
```

The goal is not a single physical crate for every host. The goal is one
contract: a plugin that behaves the same under Web and TUI should encounter the
same permission, state, resource, storage, and failure semantics.

## 1. Async host calls and long operations

Current boundary calls are synchronous request/response. That is not enough for
browser `fetch`, remote domain APIs, storage, hover/context providers, or future
cross-plugin capabilities.

Near-term design:

- Keep lifecycle exports (`load`, `render`, `render_slot`, `dispatch`) as
  bounded host-driven calls.
- Add an operation table owned by the host runtime.
- A plugin action requests long work through a host call such as
  `host.call("http.fetch", params)` or a typed WIT import.
- The host returns an operation id, writes loading state, and completes the work
  outside the plugin call stack.
- Completion re-enters the plugin through a host-dispatched action such as
  `unode.async.completed` / `unode.async.failed`, or writes a host-owned
  operation state path the screen already binds to.

Protocol conventions:

- `loading` and `status` nodes are the visible progress/error primitives.
- Operation ids are host-generated and scoped to the requesting plugin/session.
- Results are JSON values with size limits.
- Hosts apply timeouts and cancellation on route leave, plugin disable, trap, or
  explicit cancellation.

Long-term design:

- When WIT async is mature enough for both Web and TUI toolchains, typed async
  capability imports can replace callback-style re-dispatch for component
  plugins.

## 2. Resource limits

Every plugin call needs a `ResourcePolicy`. Defaults should be host-configurable
and conservative.

Required limits:

- CPU/fuel/epoch interruption for Wasmtime hosts.
- Browser-side call timeout and, eventually, Worker isolation for killable web
  execution.
- Per-instance memory cap where the runtime supports it.
- Lifecycle call timeout by phase (`manifest`, `load`, `render`,
  `render_slot`, `dispatch`).
- Host-call rate limits per dispatch and per mounted session.
- Async operation concurrency limits per plugin.
- Render output limits: JSON byte size, max node count, max depth, max children
  per node, max slot contribution count.
- Patch/write limits: max writes per dispatch and max value size per state path.

Failure mode:

- Resource exhaustion is a plugin fault, not a host crash.
- The host drops the current plugin session, marks the plugin degraded or
  quarantined according to policy, removes its slot contributions from the
  active tree, and exposes a host-owned error UI.

## 3. State namespacing

State paths cannot remain a shared global convention. Host calls must resolve
paths through the caller origin.

Recommended scopes:

- `local`: screen/session state for the calling plugin. This is the default for
  `ctx.state.set("ui.count", ...)`.
- `plugin:<id>`: private state for a plugin, writable only by that plugin unless
  a host policy grants otherwise.
- `shared:<scope>`: explicit host-declared shared state, permission-gated.
- `route`: host-owned route/query state.
- `host`: read-only host context exposed by the shell.

Slot contributions make this mandatory: injected nodes belong to the
contributing plugin, not the screen owner. Action dispatch, state writes, and
capability checks must use contributor origin metadata preserved through
normalization and IR lowering.

## 4. Crash isolation policy

A trap, panic, malformed ABI response, timeout, or resource failure must never
take down the host or unrelated plugins.

Contract:

```rust
struct PluginFault {
    plugin_id: String,
    phase: PluginPhase,
    kind: PluginFaultKind,
    message: String,
    recoverable: bool,
}
```

Policy:

- Drop the current guest session on any fatal guest fault.
- Keep the compiled module cache only if the failure is not a validation or
  provenance failure.
- Quarantine after repeated faults with exponential backoff.
- Remove active slot/shell contributions from the faulty plugin.
- Show a host-owned error surface; plugin-authored UI is not trusted after a
  fatal fault.
- Allow explicit user/admin reload.

The TUI session-per-mounted-screen model is the baseline lifecycle shape: cache
compiled modules, but treat mounted plugin sessions as disposable isolation
units.

## 5. Persistent storage

`ctx.storage` is a real product requirement, but it should not be implemented as
ordinary screen state.

Contract:

- Host-owned `StorageProvider`.
- `session` scope: survives route remounts inside a host session.
- `persistent` scope: survives app restarts.
- Keys are namespaced by plugin id and plugin version metadata.
- Reads/writes require `storage.*` permissions.
- Quotas exist per plugin and per scope.
- Values are JSON with max byte size.
- Storage APIs use the async operation model where the host backend is async.

Migration:

- Persistent data is versioned by plugin id.
- Install/update lifecycle can run migrations from old schema versions.
- Uninstall policy can delete all plugin data or preserve it behind a user/admin
  choice.

## 6. Plugin lifecycle events

Install/enable/disable/uninstall and storage migrations need an explicit
lifecycle surface.

Short-term raw ABI:

- Add one required `plugin_lifecycle(request) -> response` export in a future ABI
  version.
- The SDK macro supplies a no-op default so ordinary UI plugins do not need to
  implement it.

Long-term WIT:

- UI plugins keep the `unode:plugin` world.
- Service/headless plugins can use a separate `unode:plugin/service` world
  without render exports.

Events:

- `install`
- `enable`
- `disable`
- `uninstall`
- `migrate`
- `activate`
- `deactivate`

Lifecycle hooks are bounded calls subject to `ResourcePolicy`.

## 7. Distribution and trust

Plugin loading needs provenance, not just a `.wasm` path.

Package metadata should include:

- plugin id, name, version, ABI version, artifact kind (`raw` or `component`);
- content hash for each artifact;
- signature over manifest + artifacts;
- declared permissions and allowed origins;
- minimum host/API version;
- update channel metadata.

The host verifies provenance before instantiation. A provenance failure is a
load failure, not a plugin runtime fault.

## 8. Host conformance kit

The raw-vs-component golden test should grow into a reusable host conformance
suite.

Required scenarios:

- ABI version/export validation.
- Permission denied and missing host function.
- Host-call error envelopes.
- Malformed lifecycle output.
- Normalization and IR lowering golden files.
- Patch planning for state writes.
- Slot origin preservation.
- Async operation success/failure/cancellation.
- Resource-limit failures.
- Trap/quarantine behavior.
- Storage quota and migration behavior.

A host that passes the suite can claim compatibility with a specific Unode
protocol/ABI version.

## 9. Accessibility contract

The AST has semantic roles, but accessibility needs a systematic contract.

Needed fields and validation:

- Required labels for interactive nodes.
- `description` / help text for controls.
- `alt` for meaningful media.
- Error association for inputs and forms.
- Focus target identity through normalized node keys.
- Dialog/overlay semantics once `Overlay` exists.
- Host warnings for weak accessibility and hard errors for invalid interactive
  nodes.

Renderers translate the contract into platform-specific accessibility APIs. The
host/runtime validates the semantic data before rendering.

## 10. Form validation contract

`input` and `form` nodes exist, but validation semantics need a contract.

Recommended shape:

- Constraints are serializable data: required, min/max, length, pattern, enum,
  custom message keys.
- Form state lives in namespaced host state.
- Submit is a symbolic action.
- Synchronous validation runs in the host before dispatch.
- Async validation uses the async operation model.
- Errors are represented through `status` plus input/form error associations.

## Implementation order

1. Align docs and public authority boundaries.
2. State namespacing and origin propagation.
3. Crash fault model and resource-policy scaffolding.
4. Async operation table and callback-style completion.
5. Storage provider on top of namespacing, permissions, quotas, and async.
6. Lifecycle export/hook design.
7. Distribution metadata and provenance verification.
8. Accessibility and form validation contracts.
9. Host conformance kit.
