# Deprecated TypeScript Prototype

`ts-implementation/` is deprecated. It remains in the repository only as a
migration reference for the pre-Rust Unode prototype.

Do not add new runtime, renderer, bridge, or plugin work here.

Current web work lives in `packages/` and `examples/`, which use the intended
architecture:

```text
plugin.wasm + unode_web_host.wasm + JavaScript bridge + framework mount package
```

Before deleting this directory entirely, verify that no tests, docs, examples,
or migration notes still depend on the old TypeScript prototype.
