# unode Roadmap

This roadmap describes a pragmatic path from the current implementation to the target cross-platform architecture.

## Phase 1: Freeze the architectural language

Goal:

- make the intended architecture explicit before hardening APIs

Work:

- document the canonical goals
- document the layer boundaries
- document what belongs in core vs bridge vs renderer
- stop treating exploratory pseudo-code as final spec

This phase is what this docs rewrite is for.

## Phase 2: Define the canonical core AST

Goal:

- keep a small portable AST that both Web and TUI can implement

Work:

- identify the minimal core node taxonomy
- mark current compound nodes as sugar candidates
- explicitly keep `table` out of the core for now
- formalize keys, ids, meta, and expression support
- keep the emitted AST serializable and read-only

Success criterion:

- a plugin author can build rich screens mostly by composition, not by needing many special node kinds

## Phase 3: Introduce the real runtime contract

Goal:

- support both local reactive updates and route-driven reloads cleanly

Work:

- formalize `load()` and `render()`
- add a proper screen state store contract
- add expression resolution and dependency tracking
- add core i18n registration and translation helpers
- define explicit invalidation and refresh behavior

Success criterion:

- local state changes do not require full screen rerender
- route changes still allow full reload cycles

## Phase 4: Clarify core vs bridge

Goal:

- make `unode` truly app-domain agnostic

Work:

- keep generic built-ins in core
- keep commands, navigation, and providers in core
- move domain APIs behind app-specific bridge packages
- add method-level permission metadata

Success criterion:

- another app could reuse the core without inheriting Mugen-specific concepts

## Phase 5: Extract renderer contracts

Goal:

- make the current Web renderer a clean implementation of a generic renderer interface

Work:

- define renderer context contracts
- isolate platform-specific behavior
- keep theme/style decisions renderer-side
- adapt the current Svelte renderer to the cleaner contract

Success criterion:

- the Web renderer still works, but its responsibilities are clearer and more portable

## Phase 6: Build TUI renderer MVP

Goal:

- prove the architecture is actually cross-platform

Work:

- implement route handling in TUI terms
- map focus/navigation semantics to terminal affordances
- define media degradation rules
- support the same symbolic actions and host bridge contracts

Success criterion:

- at least one non-trivial plugin route works in both Web and TUI from the same semantic AST

## Migration heuristics

When deciding whether something belongs in the core, ask:

- can both Web and TUI render this faithfully enough?
- is this semantic, or is it mostly presentation?
- can this be composition or sugar instead of a new node kind?
- does this belong to every app, or just this app?

If the answer points to presentation or app-specific behavior, it probably does not belong in the `unode` core.

## Near-term priorities for this repo

- document the target architecture
- shrink the conceptual core AST
- preserve the current Web plugin renderer as the first renderer
- evolve `src/lib/plugins-bridge` into a clearer app bridge
- add a real local state and binding contract before attempting broad node expansion
- make AST freezing a hard invariant across plugin authoring and renderer consumption
