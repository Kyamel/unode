# mugens-sdk

`mugens-sdk` is the Mugens-specific plugin bridge SDK.

It is where app/domain concepts become capabilities that plugins can request and
call. This crate can build on `unode-sdk`, but the concepts here are not part of
generic Unode.

## Owns

- Mugens-specific plugin-facing APIs;
- domain permission names and helper constructors;
- domain UI sugar built from Unode primitives;
- bridge types that connect `mugens-domain` to plugin code.

## Does Not Own

- generic Unode AST/state/reactivity behavior;
- browser or terminal renderer implementations;
- low-level plugin memory ABI helpers;
- app runtime lifecycle.

If another application adopts Unode, it should create its own bridge SDK rather
than depending on this crate.
