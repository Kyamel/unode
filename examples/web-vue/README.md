# web-vue example

Vue 3 demo for the Unode web runtime: same counter plugin, same recipes —
`action` nodes are backed by the host's `Button.vue` through the `unode-vue`
host-slot portal.

```sh
./build.sh                # inside the nix shell (wasm targets)
pnpm install && pnpm dev  # or: pnpm build && pnpm smoke
```
