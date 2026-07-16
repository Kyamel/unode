# web-vanilla example

The framework-free web demo: no React, no Svelte, no adapter package — just
`unode-web-renderer` producing DOM and Unode's own keyed reactivity applying
patches from the wasm host core.

Why it exists: to prove the reactivity belongs to Unode, not to a framework.
`Button.ts` shows the vanilla idiom for host components — the `action` recipe
builds a real DOM button, no host-slot portal required.

```sh
./build.sh                # inside the nix shell (wasm targets)
pnpm install && pnpm dev  # or: pnpm build && pnpm smoke
```
