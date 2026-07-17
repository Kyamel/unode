---
title: Architectural Gaps
description: The systemic runtime contracts still needed for a safe generic Unode plugin system.
---

These are the missing architecture contracts that sit below product features.
They belong to the trusted host runtime, not to renderer adapters.

## Authority model

The host runtime owns plugin loading, sandboxing, capability injection,
permission enforcement, state, resource policy, storage, and crash recovery.
Renderers receive trusted IR and patch ops, draw native UI, manage focus and
interaction, and report symbolic actions back to the runtime.

## Shared host-kernel shape

Web and TUI can have different code, but should converge on one contract:

```text
PluginInstance
  -> raw ABI module or Component Model instance

HostSession
  -> PluginInstance + PermissionProfile + StateNamespace + ResourcePolicy

CapabilityRouter
  -> permission/origin/namespace validation
  -> sync host calls + async operation scheduling

CrashSupervisor
  -> traps, timeouts, resource failures, quarantine

StorageProvider
  -> session/persistent plugin storage, quotas, migrations
```

## Priority gaps

1. **Async host calls / long operations.** Use host-owned operation ids,
   loading/error state, and callback-style re-dispatch now; move to WIT async
   later when toolchains are ready.
2. **Resource limits.** Bound CPU, memory, lifecycle time, host-call rate,
   async operation concurrency, render JSON size, node count, tree depth, and
   state write size.
3. **State namespacing.** Resolve state paths through caller origin:
   plugin-local by default, explicit shared scopes only when permission-gated.
4. **Crash isolation.** A trap, panic, timeout, or malformed response drops only
   the plugin session, removes its contributions, and surfaces host-owned error
   UI with quarantine/backoff policy.
5. **Persistent storage.** Add host-owned session/persistent storage scoped by
   plugin id, gated by `storage.*` permissions, quota'd, async-friendly, and
   versioned for migrations.
6. **Lifecycle events.** Add bounded install/enable/disable/uninstall/migrate
   hooks. Raw ABI can gain a required export with SDK no-op defaults; WIT can
   later split UI and service worlds.
7. **Distribution and trust.** Package metadata needs content hashes,
   signatures, provenance, artifact kind, minimum host/API version, permissions,
   allowed origins, and update channel metadata.
8. **Host conformance kit.** Turn golden tests into a suite for ABI validation,
   permission denial, malformed output, slots, async ops, resource failures,
   crash policy, storage, and migration behavior.
9. **Accessibility contract.** Require semantic labels, descriptions, media alt,
   error associations, focus identity, and overlay/dialog semantics before
   renderers translate them to platform APIs.
10. **Form validation.** Define serializable constraints, host-side validation,
    submit semantics, async validation, and standard error display through
    `status` plus input/form associations.

## Suggested implementation order

Start with state namespacing and origin propagation, then add the plugin fault
model and resource-policy scaffolding. Async operations and storage build on
those foundations. Lifecycle, provenance, accessibility, forms, and conformance
can follow once the runtime authority boundary is stable.
