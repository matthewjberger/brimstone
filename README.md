# Boomer

An arcade boomer-shooter built on the [nightshade](https://github.com/matthewjberger/nightshade) engine. Fight through a run of distinct neon levels, clear each one to open its exit gate, then push deeper as the difficulty loops and scales. Kills drop health and ammo, so standing still starves you. The only way to live is to keep moving and keep killing. Runs natively and in the browser via WebAssembly.

## Play

- **WASD** / left stick: move
- **Mouse** / right stick: look
- **Left click** / right trigger: shoot
- **Ctrl** / B: dash (brief invulnerability, dodge through fire)
- **Space** / A: jump
- **Shift** / left trigger: sprint
- **1 / 2** / d-pad: switch weapon
- **Esc** / Start: pause
- **R** / A: retry after death

Two weapons with a real tradeoff: the shotgun is a heavy close-range burst, the nailgun is a fast spray that picks off ranged casters. Three enemies, three problems: imps body-block, swarmers rush your flanks, and casters lob fireballs you have to dash around. Chain kills to climb the score multiplier.

## Run

Native:

```
just run
```

Browser (WebGPU):

```
just init-wasm   # one-time: wasm target + trunk
just run-wasm
```

> WebGPU works in Chromium-based browsers and Firefox 141+.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
