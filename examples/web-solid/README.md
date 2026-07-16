# web-solid example

SolidJS demo for the Unode web runtime: same counter plugin, same recipes —
`action` nodes are backed by the host's `Button.tsx` through the `unode-solid`
host-slot portal (a Solid store per slot, fine-grained updates).

```sh
./build.sh                # inside the nix shell (wasm targets)
pnpm install && pnpm dev  # or: pnpm build && pnpm smoke
```
