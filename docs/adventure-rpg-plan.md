# Adventure Mode → Open-World RPG

Turn brimstone's adventure mode into a Fallout 3 / Skyrim-style streamed open-world
RPG: an effectively unbounded procedurally generated overworld you roam freely, with
towns, points of interest, dungeons/interiors, quests, loot, and combat — built on
the existing nightshade engine, reusing the current cube / billboard / prototype-texture
art (art is upgradable later; adventure mode keeps the polish level of the current game).

## Why this is reachable now

The engine already ships the three pillars of a streamed world, all callable from game code:

- **Procedural clipmap terrain** — `enable_terrain(world, seed)`, `set_terrain_height_range(min, max)`,
  `set_terrain_snow_height(h)`, and `terrain_collider_system(world, center)` to stream physics
  colliders around the player. Render clipmap LOD is automatic.
  (`crates/nightshade-api/src/terrain.rs`, `crates/nightshade/src/ecs/terrain.rs`.)
  Reference usage: `apps/sandbox/src/systems/world/terrain.rs`.
- **GPU instancing** — `spawn_instanced_mesh_with_material(world, mesh, Vec<InstanceTransform>, material)`
  plus `InstancedMesh::{add,remove,set,set_instance_tint,clear}_instance*`. Thousands of props
  become one draw call; GPU frustum/occlusion culling via `renderer_state.gpu_culling_enabled`.
  (`crates/nightshade/src/ecs/world/commands.rs`, `.../ecs/mesh/components/instanced.rs`.)
- **Async loading** — background decode workers with per-frame budgets
  (`loading_pipeline_*`, `crates/nightshade/src/ecs/loading.rs`).

And there is a **complete working streaming reference**: `apps/city` — a procedural streaming
city with chunked deterministic generation, proxy-LOD rebuilds, and streamed lights.

### The one real gap: navmesh

Navmesh is a **single global, synchronous bake** (`generate_navmesh_recast`, one
`world.resources.navmesh`, no tiling/stitching/streaming). A streamed exterior cannot
re-bake a global navmesh as the player moves.

**This limitation is turned into the core design, exactly like Skyrim/FO3:** dense ground
combat lives in bounded **cells** (dungeons/interiors/POIs), each with its own small
synchronous bake; the **exterior** overworld is traversal + sparse encounters that don't
need a baked ground mesh (flying / simple-steering AI), until we add tiled navmesh later.

## World model

- **Overworld (exterior):** streamed, effectively unbounded, seed-deterministic. Terrain
  heightfield + streamed instanced scatter (rocks, trees, ruins, grass) + streamed props /
  structures + streamed lights + POI markers. Roam freely, no walls, fog horizon, day/night.
  Sparse ambient combat only.
- **Cells (interior):** bounded locations reached by entering a POI (door / cave / gate).
  A cell = the existing arena builder (`level::build_arena`) + a synchronous navmesh bake →
  dense combat, loot, NPCs. Exit returns to the overworld at the POI. This is exactly the
  current hub-and-spoke, upgraded: the "hub" becomes a real world, the "spokes" become cells.

## Streaming architecture (port of `apps/city`, then promoted to engine API)

Coordinate scheme and lifecycle, proven in `apps/city/src/streamer.rs`:

- **Chunk grid:** `chunk = floor(pos / CHUNK_SIZE)`; Chebyshev-distance rings; camera-driven.
- **Deterministic per-chunk generation:** `seed = cx*73856093 ^ cz*19349663` seeds an RNG, so
  every chunk regenerates identically with no storage. Biome/character from layered Perlin.
- **Proxy LOD rebuild:** flatten the visible radius into one `ChunkData` of instance batches,
  rebuilt only when the camera crosses N chunks. Distance rings drop detail (subsample scatter
  far out, props/roads/markings/lights only within tighter rings).
- **Multi-tier LOD with hysteresis** (from the old demo): load at `radius + forward_bias`,
  unload only past `radius + bias + 2` to stop thrash; 0.3s fade to hide pop.
- **Per-frame entity budget** (~a few hundred spawns/frame) so streaming never hitches — this
  is the amortized-spawn answer to the engine's synchronous entity spawning.
- **Light + collider streaming** near the player; terrain colliders via `terrain_collider_system`.

## Content / worldgen (brimstone side)

- **Biome noise** (like the city's district noise) drives terrain height params, scatter palette,
  enemy palette, POI density, fog / atmosphere / lighting. Biomes fit the theme: plains, forest,
  ash-wastes, hills, neon-ruins.
- **POIs** placed deterministically per region: towns (safe, NPCs, shops, quests), dungeons
  (combat cells), ruins, camps, shrines. Optional roads/paths connecting them.
- **Reuse current art:** prototype textures / cubes / billboards (`art.rs`, `textures.rs`),
  existing enemies / weapons / fx / HUD unchanged, so the overworld matches the game's look.

## RPG systems (layered on, keeping current quality)

- Player movement/combat/weapons — reused wholesale in the overworld.
- NPCs / dialogue / quests already exist in `adventure.rs` — expand into quest log, dialogue
  trees, objective markers (the objective compass already exists in the HUD), journal.
- Inventory / loot / vendors (merchant NPC exists), leveling from existing kills / score / stats.
- World map + minimap (old demo has the pattern), fast travel to discovered POIs.
- Day/night (engine), weather; world-state save/load (discovered/cleared POIs, quest state) —
  extends the existing campaign persistence.

## Milestone roadmap

Each phase ends at a committable, runnable state. Engine (nightshade) vs game (brimstone) noted.

**Phase 0 — De-risk & wire (game + build):**
- Run `apps/city` to see streaming live; study `apps/sandbox` terrain usage.
- Point brimstone at local nightshade via a **path dependency** (`../nightshade`) so engine and
  game co-develop. Confirm brimstone still builds (native + wasm) against the local engine.

**Phase 1 — Walkable streamed overworld (kills the constraint):**
- New adventure "overworld" state: `enable_terrain(seed)`, height range, `terrain_collider_system`
  each frame, fog horizon, sun/day-night. Player spawns and roams real terrain, no walls.
- First streamed scatter: minimal chunk streamer in brimstone (rocks/trees via instanced geometry
  using existing materials) with proxy LOD + hysteresis + per-frame budget.
- Adventure "open" loads the overworld instead of the town arena; existing controller/HUD intact.
- Runnable: walk a big procedural terrain world.

**Phase 2 — POIs & cells (Skyrim structure):**
- Deterministic POI placement; POI markers (stretched-cube gates per the billboard rule) with the
  objective compass pointing at the nearest.
- Enter POI → load bounded cell (arena builder + sync navmesh) → combat/loot → exit to overworld at
  the POI; persist "cleared" state. One town hub (existing NPCs/merchant/quests) + several dungeons.

**Phase 3 — Engine packaging (the "update nightshade" deliverable):**
- Extract the streamer into a reusable nightshade module / api helper (`WorldStreamer`) taking a
  content-generator callback, with LOD tiers, hysteresis, budget, light/collider streaming.
  Refactor brimstone to consume it. Add public **LOD-mesh registration** API (fills the
  internal-only gap so distant scatter can swap to cheaper meshes).

**Phase 4 — Overworld combat & tiled navmesh (hardest engine gap):**
- Add tiled / streamed navmesh to nightshade: per-chunk Recast bakes keyed to loaded
  terrain/obstacles, stitched or region-queried, baked off-thread via the loading-pipeline pattern.
  Enables ground enemies to roam the exterior. (Interim: flying / simple-steering ambient AI needs
  no navmesh.)

**Phase 5 — RPG systems:**
- Quest log + dialogue trees, objective markers, journal; inventory / loot / vendors; leveling /
  perks; world map + minimap + fast travel; weather, more biomes; world-state save/load.

**Phase 6 — Polish:** LOD fade tuning, occlusion/draw-distance/memory budgets, per-biome audio ambience.

## Risks & mitigations

- **Exterior navmesh** (biggest gap) → cell-based combat first, tiled navmesh in Phase 4.
- **Terrain visual fit** with the blocky look → judge in Phase 0/1; fall back to instanced
  cube-chunk terrain (city pattern) if the clipmap clashes. Art is upgradable later.
- **Perf at scale** → instancing + GPU culling (designed for ~1M instances) + budgeted streaming;
  proven by `apps/city`.
- **Dependency/version** → path dep on local nightshade; keep the wasm build green (loading budgets
  and worker counts differ on wasm — verify streaming there too).
- **Dual maintenance** (engine + game together) → path dep handles it; commit per milestone.
