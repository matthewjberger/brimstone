# Boomer

A boomer-shooter vertical slice built on the [nightshade](https://github.com/matthewjberger/nightshade) engine. Clear an arena of billboard imps across three waves with a hitscan shotgun. Runs natively and in the browser via WebAssembly.

## Play

- **WASD** / left stick: move
- **Mouse** / right stick: look
- **Shift** / left trigger: sprint
- **Space** / A: jump
- **Left click** / right trigger: shoot
- **Esc** / Start: pause
- **R** / A (on the end screen): restart

Grab the floating medkits and ammo boxes, survive the waves, clear the arena.

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
