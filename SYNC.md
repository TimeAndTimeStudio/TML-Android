# Sync with Horizon (temporary, delete after the merge lands)

Upstream commit merged: `31f83499b` (2026-11-02, horizon-rust/horizon#7585). Base `f8988d5a7`.
Android before the merge: `73fb969be`. Wasm removal, files only: `30344a60b` (does not build).

The merge commit `212b96cb7` holds its conflict markers on purpose; each group below was resolved in
a commit of its own. Part of the conflicts came resolved from the older `sync-horizon-3` worktree,
read only and carried forward with 3-way merges (the mod crates were left out).

How to work and tools: `sync4/` in the session notes folder
(`%APPDATA%/Claude/side-session-notes/local_4c49e3d5-8a78-4799-b072-15ba12d5ea8a/`): `BRIEF.md`,
`COMPILE.md`, `FOLLOWUPS.md` (what fell, in detail), `vm.sh MODE` (check, all, android, clippy,
clippy-android, fmt, test, raw CMD). On the VM: `~/sync4`, `~/sync4-target`, `~/sync4-apk.sh`.

## Steps
- [x] A. Mods: `8f18fd9d6`
- [x] B. Render and shaders: `26f8ed428`
- [x] C. Client gameplay: `2315ac2db`
- [x] D. game.rs, title, Cargo, CI: `3bbeb3a20`
- [x] E. Desktop builds (`cda4b7629`); Android builds, hooks in `horizon_client::android`, Dev APK (`7a5fb6925`)
- [x] fmt
- [x] Client opens on the VM (Xvfb, software Vulkan, new world, no mods or pack)
- [x] Tests: 5495 passed, 0 failed, 104 ignored
- [x] Clippy desktop and Android, `-D warnings`
- [ ] Owner tests the APK; merge into android

## What fell
- Wasm mods and their API: horizon-mod-sdk, starling-mod-sdk, starling-mod-cli, mods/official
  (far-view, fps-overlay, keystrokes, minimap, recipes, zoom) and the SDK examples.
- Create: own block states (`states`, `own-states.json`) and so placement orientation and belt
  funnels (`belt_funnel_for`); `fuel` on items; the support and occlusion part of `properties.shape`.
  The local .hmod data has to move to `state_properties` and `components`.
- Create New World's "Mods & Data Packs": upstream's Data Packs screen stays; a new world plays with
  the running mods; the phone's `datapacks` folder choice.
- Stack tooltip grids; Starling's remote item displays (upstream's `text_displays` instead).
- Failed shader experiments `HORIZON_PACK_SKY_LAST`, `HORIZON_PACK_VRS`, `HORIZON_PACK_HALF`.
- Item grids in tooltips (`StackTooltip`), the Config button and summary line on the Mods page.
- Android: `learn_layout` does nothing, the splitscreen pad router is a stub.

## Not checked
Nothing ran on a phone. On the VM the client ran without mods and without a shader pack: Create,
shader packs, touch, custom buttons, the setup download and native mods builds were not run after
the merge.

## TML fixes after the merge (not upstream)

- Fix the resource pack screen's Done/Escape: the shader-pack removal
  (`4b93505cc7`) gutted `OptionsUi::back`'s `match page` block, so leaving
  the page never ran `PackSelectionModel.commit`. The bundled `TML.zip`
  (turned on once by `preparer::put_tml_ui_pack`) therefore could not be
  turned off. Restored the `Language`, `Packs` (commit), `Skin`,
  `CustomDraw` and `CustomButton` arms from the pre-removal source, minus
  the `ShaderOptions` arm; the possible commit (`player_pack_ids`), the
  skin-name commit, the canvas save and the custom-send commit come back
  with them. Tests: `leaving_the_pack_page_commits_the_selection` added.
- Fix the frame rate drop: `b6388cf2b8` had made the GPU-culling trial aggressive
  — it ran from 5 s and kept indirect multi-draws with occlusion culling when
  the read-back hid as little as 6% of the solid indices, even when the frame
  time got worse; and it lowered the Android test bar (`PHONE_MIN_SECTIONS`
  96 -> 64, a `PHONE_MIN_HIDDEN_SHARE` of 0.06). On Android the baseline is a
  call a section, which `indirect_draws_wanted` measured faster than indirect
  multi-draws, so keeping the pair on a hidden share alone left most phones on
  the slower path. The trial is back to the platform's own gate: it starts after
  25 s, runs for 10 s, and keeps the pair only when the frame time held within
  2%; `min_sections`/`min_share` on Android are 96 and `MIN_SHARE` (0.1) again
  (`Renderer::occlusion_stats` stays, for the trial's log).
- Ship the APK from a `ship` profile (release's cross-crate thin LTO, 16 codegen
  units, `incremental = false`) instead of `fast`, which leaves cross-crate
  inlining between the game crates on the table on every frame.

## Ported after the squash (2026-11-23)

The fork's history is a single squashed root (`1beb91ea75`), so `git merge upstream/main` has no
common ancestor to merge from. Stable upstream changes that matter to phones are ported by hand:

- `ab44288662` — cloud mesh rebuilds through a precomputed L2 cloud mask (+2.07x CPU).
- `2d43808325` — shader-pack sky visibility switches (`sky`, `stars`, `sun`, `moon`).
- `95c4d72ce9` — the sky gradient and sunrise fan follow the same view pose (bob/spin) as the
  world, so the sky no longer stands still under a bobbing camera.
- `e55ba16038` and later — TML's own shader cache, resolution-scale and default-pack changes
  stay on top and are not upstream.
- `15c1eec6c6` — shader-pack expressions and `#if` conditions have a nesting limit.
- `58d1bd9ce5` — a continued `block.properties` value continues past a preprocessor group.
- `a95d0770af` — numeric promotion stays local to each shader-pack operation.
- `ca9f24c304` — classic GLSL parsing/lowering depth is bounded (naga).
- `8365a5f59e` — section arena pages grow by half, not double (less video memory).
- `e349aeccea` — chunk lighting BFS reads the sky layer directly and packs its bits (-7.9% CPU).
- `f43817376a` — upload staging ring is bucketed and trims when idle (-13 to -25 MB RSS).
- `6d40db2be5` — shadowtex0/1 read through a plain sampler filter linearly as Iris does, which
  removes the striped shadows on distant ground; a pack's `shadowtexNNearest` keeps nearest.
- `d652ec2f1b` — classic packs' terrain is drawn from the camera rounded to 1/65536 block, as
  Iris does, so shadow-map texel ties no longer flip and stripe the ground.
- `b45d73de43` — weather columns outside the view frustum are not built or uploaded.
- `322774f158` — storage images may declare a compatible qualifier (`rgba16` over `RGBA16F`);
  the naga module is rewritten to the texture's real format, so IterationRP's bloom no longer
  leaves a white frame.
- `8e0b8ff5a5` (part) — `LightmapKey`: the 256-texel lightmap recompute is skipped when its
  inputs did not change.
- `8e0b8ff5a5` (part 2) — skip hand/crosshair global uploads when there is nothing to draw, and
  skip the sunrise fan upload when the sky is not visible.
- `8e0b8ff5a5` (part 3) — `AnimatedTickCache`: a tick's animated sprite layers are walked and
  dispatched once. Later frames of the same tick skip the walk; a culled frame still animates a
  layer that has just come into view, and a title frame marks every layer done. The commit's
  `has_modes` guard is omitted (this tree has no Polytone `mode`s), and its HUD-sprite skip is
  moot: `HudRenderer::sprite_animations` has no producer here.
- Java removed from the app: no runtime download, no on-phone preparation, no exporter bundle
  or `java`/`javac` launchers in the APK. The first-run screen imports the `resources` folder
  prepared on a computer (zipped) and keeps sign-in/offline; nothing is fetched from Mojang.
- Storage: dead space is freed at start — `minecraft/` (the phone no longer prepares), the old
  Java preparation folders, downloaded update APKs, staged imports, and a redundant private
  `resources` copy; old resource-pack overlays beyond the one in use are dropped. The log prints
  what the app's own files hold, folder by folder.
- Android: `--resources` follows the game folder (shared storage when allowed), so moving the
  resources out of app storage no longer hides them from the game.
- TML Visuals Lite: two cloud sheets (the vanilla height and a higher, slower one), drawn over
  terrain where they are in front of it, to the horizon instead of only overhead.
- Shadows: a `shaders/` pack with its own shadow pass hides vanilla's round entity shadows too;
  a pack whose shadow programs did not load says so when it loads.
- Shaders: Iris's `mc_hasGlint()` and `mc_sampleGlint()` are supplied to every program. Mellow
  v3.5 calls them in its gbuffers programs; without them 17 programs (terrain, water, entities,
  hand, weather) were skipped and the pack drew with a plain texture and no shadows.
- Shadows: vanilla's round entity shadow is left out whenever any shader pack runs (not only a
  pack with a loaded shadow pass), so it can no longer stay on under Mellow, Fantasy or others.
- Setup: the import button is gone. The card says to copy the prepared `resources` folder into
  the TML folder and offers "Try again" (the game folder is looked at again). No empty
  `resources`/media folders are made at start, and an empty one left by an older build is
  removed, so a folder with no files no longer looks like lost resources.
- Resources zip: a `resources.zip` (or any top-level zip holding `horizon-resources.json`) left
  in the game folder is unpacked into `resources` at start — and unpacked again when the files
  are missing or the zip was replaced. "Try again" also unpacks it, so no file picker is needed.
- Bundled resources: a `packaging/resources.zip` (or `TML_RESOURCES_ZIP`) is stored in the APK's
  assets; when the phone has no resources, it is streamed out and unpacked, and unpacked again
  whenever they go missing. Clearing app data cannot lose it. CI takes the zip from the
  `TML_RESOURCES_URL` secret.
- On-phone preparation is back: the app downloads the Android Java runtime, runs the exporters
  and prepares `resources` again (no computer needed), then writes `resources.zip` beside them.
  The zip is re-unpacked if the folder is lost. Fetched `minecraft` files stay and move to
  shared storage instead of being deleted.
- No `resources.zip`: it doubled the space the game used, and preparation on the phone can
  simply run again when the resources are lost. The zip generation, the bundled APK copy and
  the automatic unpacking are gone.
- Fetched files live only in `TML/minecraft`: without All files access the setup asks for it
  (and offers the restart that applies it) before anything is downloaded or prepared, downloads
  refuse to start, and a private `minecraft/` left by an older build is removed.
- Setup progress: every step after the sounds now has a line — verifying the prepared files,
  putting them in the TML folder (n/total while the copy runs), touch buttons from Mojang, then
  control textures — so preparation no longer sits silent on the last sounds line for minutes.
- Setup progress, rest: the resource swap reports too — removing the previous game files (n/total,
  the backup a stopped run left in the TML folder, minutes on shared storage), moving the files
  (n/total), and cleaning up the work files. "Putting the game files in place" no longer covers
  minutes of silent deleting.
- The game files stay in the app's own storage again: `resources` and `minecraft` are never
  copied into the TML folder (that cross-filesystem copy was minutes per preparation and looked
  hung). Preparation installs with a rename beside its kit, the fetched files download into the
  app's own `minecraft`, and the TML folder keeps only the player's files (saves, packs, options).
  The All-files-access gate is gone: nothing needs it to play. A copy an older build moved into
  TML/resources or TML/minecraft is no longer read; the log says how many MB it can free.
- Shaders: naga's GLSL nesting limit is 128 (was 32): Solas v3.9's `pow32`/`pow24` helpers are
  chains of 32 multiplications and every program was refused ("GLSL nesting limit exceeded").
  The pack's programs are translated on a 16 MiB stack, which leaves room for it.
- Shaders: every program logs "translating"/"translated in N ms" and every pass logs "building"
  before the driver sees it, so a pack that takes the app down (Photon) leaves the program it
  was on in the log.
- TML Visuals Lite: the cloud sheets drift at about 0.6 blocks a second (they were scrolling at
  178 blocks a second, the whole sky racing past), and the detail wisps drift with them.
- Shaders: on a GPU without vertex-stage storage images (Mali-G57; `VERTEX_WRITABLE_STORAGE` is
  absent) the vertex stage's `imageStore`/`imageAtomic*` calls are dropped before naga sees them.
  Solas v3.9 voxelizes from its shadow vertex stage and Iris (OpenGL) allows it; wgpu refused the
  pipeline ("Visibility flags don't include the shader stage"), so the pack loaded without
  shadows. Its voxel lighting now reads zero; everything else runs.
- Shaders: the pack thread's stack is 32 MiB (was 16): naga's SPIR-V writer recurses into a
  program's deepest expressions while a pipeline is made, which is where Photon's terrain died.
- Shaders: a pipeline refused for that reason now says so in words (a storage buffer or image in
  a vertex stage this GPU cannot bind).
- GPU culling is on by default on Android (`HORIZON_GPU_CULL=0` for A/B): the boxes are tested
  in the fragment shader there, the way Metal and DirectX 12 do, because Adreno's Vulkan driver
  left sections out for a frame with the early depth test (upstream `a09d036f50`). Tiled GPUs
  (all phones) skip the test below 1600 sections. A device that cannot run it says so in the log,
  with which capability is missing.
- Shaders: a pack whose build took the app down (a driver dying in `create_render_pipeline`,
  Photon's `gbuffers_water` on Mali-G57) is turned off at the next start instead of loading into
  the same crash; picking it again in the settings tries once more. The pack thread's stack is
  64 MiB.
- Shaders: the crash marker is read once a run, before the first build. Read every frame, it saw
  the marker the build under way had just written and turned good packs (Solas, TML Visuals Lite)
  off mid-build.
- Culling, measured: every 20 s the log says how many sections and shadow casters were drawn and
  with how many draw calls, and whether occlusion culling ran — the A/B is `HORIZON_GPU_CULL=0`
  in horizon-args.txt.
- Shaders: the Ender Dragon is marked for a pack's `gbuffers_entities` like the mobs are (its
  geometry is made outside `entity_geometry`, which marks them), and its eyes and death rays —
  additive geometry the pack never drew — join the entities as translucent draws when the pack
  draws entities. With Solas the dragon was invisible: the pack's deferred pass painted over the
  geometry Horizon drew on its own.
- Shaders: end crystals and thrown eyes of ender are marked for a pack's `gbuffers_entities`
  like the dragon's geometry is; a pack's deferred pass was painting over them (the Eye of Ender
  was invisible with Solas).
- TML Visuals Lite: clouds get a third, turned octave, a density that thins at the billows'
  edges, softer coverage and a shaded face from the noise towards the sun.
- TML Visuals Lite: `TML_DIMENSION`/`TML_OVERWORLD`/`TML_NETHER`/`TML_END` are defined for every
  pack's programs. In the Nether the cloud sheet is gone and embers drift through warm air; in
  the End a black hole hangs over the island — a black disc with a hot ring and a violet halo.
- Culling, measured properly: the line is written after the frame's draw calls are counted, and
  it says what the last test hid — boxes hidden out of those tested, and the share of solid
  indices kept out of the passes.
- TML UI (`TML.zip`): turned on for a new install only. After that the player's pack list is
  left alone, and a packaged pack the player deleted stays deleted — the zip was being put back
  and the pack re-added to `resourcePacks` on every start, so it could not be removed.
- Culling, really on now: occlusion culling draws through indirect multi-draws, and Android keeps
  those off (they measured slower on an Adreno 710), so the test never ran — the log said "hid 0
  of 0 boxes". A phone's threshold (1600 sections, from Metal) also stopped it. Android now tries
  the pair for ten seconds after the world settles and keeps it only if the frame time holds
  (`game.rs trial_gpu_culling`), with the threshold at 96 sections, and the log says which way it
  went.
- The draws trial now finishes: it returned early once the trial itself turned indirect draws on,
  so the ten-second comparison never ran and a slower choice stayed on. It also stays out of the
  way while a pack draws terrain (culling cannot reach a pack's own terrain programs). A fresh
  renderer's hidden share is 0 (was 1, "everything hidden"), and the twenty-second line says "no
  test result back yet" instead of "hid 0 of 0 boxes, 100%".
- Shader packs and occlusion culling: a pack draws terrain with its own program, so the test now
  reaches it by skipping the sections the last result found hidden (a frame of latency, no
  indirect draws), with the box test run after the pack's terrain pass on the depth it drew.
  Opt-in for now (`HORIZON_PACK_CULL=1` in horizon-args.txt): a pack's reflections or volumetrics
  may want what the camera cannot see.
- HUD: the vertex buffer is kept from frame to frame (`HudBatch::new_with`), as upstream's
  reusable HudBatch does. A fresh vector grew back to the same size every frame and the whole
  batch was copied into the upload buffer again each time.
- OIT: the feature stage is skipped whole when a frame has nothing for it (no text, particles,
  items, entities, weather, border or lightning in the translucent layers) — one pass fewer in
  the common frame (part of upstream's OIT coalescing).
- HUD, more: the batch's glyph patches, blend runs and inline profiles are kept from frame to
  frame too (`HudReuse`, `HudBatch::into_reuse`), not only its vertices.
- Mesher: the ambient-occlusion samples read one compact byte a block state (`lighting::AoInfo`,
  upstream #20199) instead of the whole `Info` (a few hundred bytes of table that then stays in
  cache); a state that cannot be packed falls back to `Info`, and the table is empty then.
- TML Visuals Lite's terrain is occlusion culled by itself (the pack reads nothing the camera
  cannot see): the client turns `Renderer::set_pack_culling` on for the packs it ships, and other
  packs still opt in with `HORIZON_PACK_CULL=1` (`=0` turns it all off).
- Occlusion culling now covers every shader pack's terrain by default (Solas included): the
  sections it skips are the ones behind other geometry, which no program can see either. TML
  Visuals Lite is no longer a special case; `HORIZON_PACK_CULL=0` turns it off.
- Regression: the indirect-draws trial could start while a pack was reloading (its choice had
  briefly cleared), and once the pack was back the trial returned early — leaving indirect draws
  on, which Android measured slower, for the rest of the run. A pack taking over now resets
  section draws to the platform's choice (`Renderer::reset_indirect_draws`), and the trial ends
  there.
- Shaders, dimension folders: a program a `world0`/`world-1`/`world1` folder does not hold falls
  back to the base `shaders/` one, as Iris reads them (Solas' End folder has no
  `gbuffers_weather`, so rain drew with `gbuffers_textured` there). Compute stages fall back too.
- TML Visuals Lite: the Nether has no sky again — its composite drew a sky past the fog there
  (sun, stars, the lot); the game's own colour stands, with no sky coverage.
- Pack culling: back to the packs TML ships by default (a third-party pack may read reflections
  or volumetrics, where a skipped section shows as missing geometry); `HORIZON_PACK_CULL=1`
  turns it on for any pack. The box test runs every other frame, a result outliving a frame.
- TML Visuals Lite, clouds as an option: `CLOUD_STYLE` 0 draws vanilla's own cloud mesh with the
  pack's shading (the default: cheapest, and what most players want), 1 one flat layer of the
  pack's, 2 the two soft layers. With 1 or 2 the game's mesh is discarded as before; with 0 the
  composite's cloud code never runs.

## Ported after the shader-pack removal (2026-11-24)

Ported by hand from `upstream/main` (no common ancestor to merge from); each is a
self-contained performance change that applied to this tree:

- `326d390035` — `BiomeZoom::corner_jitter` shares the X and XY prefix mixes.
- `0a4fe8c212` — `PackedStorage::unpack_into` has fixed-width fast paths (4-9, 15 bits).
- `5c5e6b1651` — `VoxelShape::join_is_not_empty` merges indices on the stack, heap only past 16 cells.
- `b17d429e33` — bulk section-major `lod::summarize` (7.22x upstream): uniform light sweep,
  resolved kept-states, one boundary read per section.
- `5db9bc3f45` — inline single-box `VoxelShape`, O(1) collide fast path (the mesher's
  `block_collision_scan.rs` keeps this tree's older form, which already uses `shape.moved`).
- `d41da14ca2` — `terrain.wgsl` RGSS/vertex ALU: `filtering_uniform` packs the sprite width,
  single-texel mip level and `log2(width)`, `rotated_grid` takes its mip level from squared
  gradients with a single-texel early-out, `sample_rgss` early-outs on squared derivatives.
  Adapted to this tree's terrain.wgsl (no `sprite_at`/`TerrainWire` split here).
- `1fd5f925a8` — precomputed `BakedModel` `CullFlags` (`catalog::CullFlags`) and stack-backed
  `model_slots_at_small`; the mesher tests only the cull directions the parts use and skips a
  block with nothing visible and no uncullable face. (Its `AoInfo` half was already ported.)
- `55aa15efeb` — `SamplerContext` recycles its cached and slice-plane `Vec` buffers across `reset`.
  (The upstream test additions need the later `set_lattice`/`with_identity`/`sample_needed`
  sampler refactor; they are left out.)
- `6dbd04fc1a` — `sweep_intersected_blocks` keeps visited positions in a stack array (24 slots),
  spilling to `FxHashSet` on long sweeps.
- `64baac04dd` — step-up candidate heights collect into a stack array (8 slots), heap past it.
- `3c678f8923` — `Beardifier::sample_volume` scatters per piece inside provably-safe bounds
  (bit-identical sums), falling back to the full scan otherwise.
- `8060e1ff24` — a far tile requested by several players is sampled once and shared (`lod_net/cache.rs`).
- `b6562f83a8` — `PalettedContainer::get_run` fills 16-cell 4-bit and non-straddling 2-cell runs
  from one word each.
- `bc6bc483a8` — wide JSON objects index their keys with an `IndexMap` (order-preserving, O(1)).
  (The upstream test additions need `parse_prefix_strict`, absent here.)
- `7cc881c0e3` — `try_resolve_step_movement_into` reuses one shape vector across the two step
  queries; `world_movement` keeps a thread-local `Vec<Collider>` for it. Adapted to this tree's
  `Collides`-generic movement (upstream uses `Borrow<VoxelShape>`).
- `19c9350647` — Arabic shaping consumes a run of following marks in one linear pass.
  (The upstream origin-preservation test needs `Span::origin`/`Text::untrusted`, absent here.)
- `576b17b8a9` — `LevelTicks::begin_tick` keeps each due chunk's priority, sub-tick order and
  deadline in a `Heap<DueChunk>` instead of a `Heap<i64>` re-reading the live head per comparison;
  its cached-head equivalence test is kept and passes.
- `7f693e21b4` — `Behaviours` resolves its fixed hot-path block tags (fire, lava fire spread,
  hanging signs, speleothems, nether portal frame) once at load, on both the source and cached-table
  paths, so hot paths no longer look them up by name.
- `746f280e8e` — `BlockCollisionScan` classifies a non-full block's moved shape with
  `fast_box_overlap` (exact, zero-allocation) so shapes that cannot overlap the query box are
  skipped without moving them, and reuses a per-thread moved-shape cache keyed by position and
  `VoxelShape::same_identity`. `Replay::best` runs its last candidate on `self.body` directly
  instead of cloning it. Adapted to this tree's `Collider`-based scan (upstream caches full cubes
  via `moved_block`, expressed here as `Collider::Block`).
- `150a1013d2` — a generator worker compiles the world's surface rule once, folding each chunk's
  possible-biome decisions, instead of rebuilding it per proto chunk (`build_surface` and
  `apply_carving_mask`); adds `MaterialConditions::add_shared_rule` / `apply_carving_mask_shared`.
- `635586ce1b` — canyon width buffers stay on the stack for vanilla heights and spill to the heap
  past 1024; the carving mask caches each carver's large-feature seed constants across the 17x17
  origin neighbourhood; `apply_carving_mask_shared` defers the per-cell top-material worker until
  exposed dirt needs repair, when the shared rule has no eager density bindings. Adapted
  `has_eager_density_bindings` to this tree's `CompiledRule::binds_densities` (no eager/point
  distinction here); the upstream test modules that need absent sampler APIs are left out.
- `a03f108dbc` — `Chat::add_lines` wraps a message into a capped `VecDeque` and splices the
  survivors in front once, instead of inserting every line at index 0 and shifting the whole list
  per line; the scroll length uses the uncapped running count so focused scrolling matches the old
  list length (`Chat::scroll_by` split into `scroll_by_len`). Adapted to this tree's `Line` (no
  `message` field); the upstream test's `Message::wrapped` assertions are replaced by an
  equivalence test against the old prepend-then-truncate reference (passes with the other 22 chat
  tests).
- `12cd1ed563` — `LerpSampler::sample_volume` hoists the `Values` enum dispatch out of
  the per-cell loop (one match per volume, four constant/buffer combinations), and
  `IntervalSelectSampler` classifies every cell's branch once in `classify` (at most 256
  branches; the selected index kept in a stack array up to 4096 cells) while building the
  per-branch needed masks in the same pass, so the gather no longer rescans thresholds.
  `RangeChoiceSampler` and `LerpSampler` also gained their needed-mask paths
  (`sample_choice`/`sample_lerp` with `needed_where`), and `RangeChoiceSampler::choose`
  does the conditional store with a bit mask so it vectorizes (it was a branch per value).
  Adapted to this tree: the sampler `needed`-mask refactor landed with this port
  (`sample_needed`, `sample_with`, `usable_mask`, `needed_where` in `density/mod.rs`), so
  the whole file is upstream's post-image; there is no TML-only code in it.
- `46e0a6942e` — a world's dimensions share one read of the structure mob data. The finalizers of
  one root take the same `Arc<OnceLock<Option<Data>>>` from a `Weak` registry (retained only while
  a finalizer holds it), so the overworld and the Nether read the files once instead of once each.
  Adapted to this tree's `StructureMobFinalizer`, which has no `item_data`: cells are keyed on the
  root alone.
- `c415b158fe` — `fluid_base_tick` returns early when `MotionWorld::is_dry_air_fast` proves no fluid
  can change this tick: the player's fluid flags, state and deflated bounds (plus the eye cell) are
  scanned and every cell must be `FluidKind::Empty`, with a bounded block count. The upstream
  daily equivalence test needs test fixtures absent here and is left out.
- `92abe36aec` — `PalettedContainer::compact` and `BlockSection::compact` unpack the packed storage
  once and remap cell indices through a first-use table (`remap_first_use`), hashing each distinct
  palette slot at most once instead of `get`/`set` per cell. Adapted to this tree's storage-based
  `PalettedContainer`, which has no `aliases` field, so the `aliases = None` line is dropped. The
  upstream test's `last`-cache assertion is dropped (a benign hint) and replaced by a cell-by-cell
  comparison; the test runs inline (this tree has no `chunk/tests.rs`).
- `4ac255c284` — `write_region_header` builds both header sectors in memory and writes them in one
  call, shared by `RegionFile::write_header` and `RegionWriter::finish`, and a chunk's length and
  compression byte are written in one `write_all`. The `region_write_bench` example is included
  (its `nbt` import moved to `horizon_world::nbt`).
- `ec0648aba7` — `tick_sniffer_effects` returns before listing players when no live sniffer
  exists in the level. The effect bodies already skip non-sniffers, so the guard only avoids the
  player list and the loop. Adapted to this tree's `game/sniffers.rs` (upstream's later
  `game/mobs/sniffers.rs` path).
- `debef4584c` — `tick_remote_entities` builds the orb `followed` list only when the client has
  experience orbs, and the cat `sleepers` list only when a cat is in the level. Adapted to this
  tree's `game/remote.rs` (upstream's `game/network/remote/tick_entities.rs`).
- `9609bc8ccf` — selector option keys borrow the reader's text with `Cow` instead of always
  allocating, and `parse_internal`/`into_parsed` avoid the final `Selector::clone` on the public
  path; `can_use` becomes `option_usable`, which also rejects unknown keys so the separate
  `OPTIONS.contains` check goes. This fork has no mod system, so the `selector_tokens` lookup the
  commit also guards is not present to adapt.
- `dd3ed413f2` — `pack_words` assembles one 64-bit word at a time instead of locating and
  read-modify-writing each slot, and `unpack`/`unpack_ids_mapped`/`unpack_ids` get const-generic
  fixed-width arms (4-8, 15-16) with immediate shifts. The tree already had `unpack_mapped_fixed`
  after `unpack_ids_mapped`; that duplicate is folded into the helpers block the patch adds before
  `unpack`. The absent `paletted/tests.rs` and the CI ignore lists are dropped.
- `8197ba2363` — Horizon's new `third_party/naga/src/back/bounded_loop.rs` proves a SPIR-V
  loop finite (a local integer counter only its own step changes, compared with a bound it cannot
  wrap past), so the SPIR-V backend leaves out its `force_loop_bounding` counter and exit and the
  driver's compiler sees the trip count; `gfx.rs` reads `HORIZON_SHADER_LOOP_PROOFS=0` once to
  bound every loop again for A/B checks. This tree has no `crates/horizon-shaders/tests/it/`, so
  the upstream harness test is dropped; the pass, its `back/mod.rs` registration, the `spv/block.rs`
  gate and the `THIRD_PARTY_NOTICES.md` note are ported.
- `92d8bf33e1` — a geometry stage whose every `EmitVertex` sits in its own `for (int i = 0; i < 3;
  i++)` strip loop draws one triangle per loop, so `combine()` bounds the instance count with
  `emission_loops()` instead of `max_vertices - 2`. This tree has no `array_constructors` module,
  so `matching()` is added locally in `geometry.rs` (upstream only made its `array_constructors`
  copy `pub(super)`). The two strip tests are ported.
- `e0bbdc1e25` — an open dialog keeps a `LayoutCache` keyed on the screen size, the font's
  glyph epoch and the text direction, so `dialog_layout` returns a shared `Layout` instead of
  re-wrapping its body every frame; `Part::Line` holds an `Arc<Text>` so copying the layout or
  placing the body shares each line's text, and `line_style_at` uses `Text::resolved()`. The two
  layout-cache tests are adapted to this tree's `Font::load(path, ...)` and 5-arg `line_style_at`;
  the upstream text-direction test needs `lang::isolate_publication`, absent here, so it is
  dropped.
- `d9b8e7bef2` (partial) — the TrueType per-char `HashMap<char, GlyphId>` is replaced by a
  compact canonical table (one `Option<char>` per glyph ID from `codepoint_ids()`) plus a sorted
  override list for the rare multi-subtable case, and `TrueType::glyph_id` reads it; the
  `resolve`/`provider_advance` call sites use `glyph_id`. The oversized-glyph-fallthrough half
  of the same commit needs `metrics_only`/`fishy_advance`, absent here, so only the compact-table
  half is ported. The exhaustive BMP lookup-equivalence test is adapted to this tree's
  `load_true_type(root, def)`.
- `fd3b4ff6a3` — a chunk decoder thread now calls `chunk_decoder_current` and stays at the
  ordinary priority below the frame, unless `HORIZON_DECODER_PRIORITY=1` asks for the old raise
  (a parsed-once switch, with a portable parse test and the Windows priority A/B). The
  thread-swap hunk is adapted from upstream's `game/network/remote_chunks.rs` to this tree's
  `game/remote_chunks.rs`.
- `07147ef4af` — `publish_boss_bars` returns before building viewers when no boss-bar source
  exists and nothing has been sent, `boss_viewers` reserves for the host plus its guests,
  `diff_boss_events` returns early for a viewer with no desired bars and no sent map, and an
  emptied sent map is removed so the skip can resume. This tree has no
  `dimension_type().has_ender_dragon_fight`, so `dragon_bar_possible` (dragon.rs) mirrors
  `dragon_boss_progress`'s End/loaded/not-won test and `has_raids` (raids.rs) exposes
  `raiding.raids.map.is_empty()`; the private fields live in sibling modules.
- `1bde596b8d` — `sculk_step_on` and `sculk_step_on_player` return while the sculk index is
  empty, and `sculk_stand_on` takes a slice so the player path borrows a one-element array
  instead of a Vec. Adapted to this tree's `game/sculk.rs` (upstream's `game/blocks/sculk.rs`).
- `fdf87ca16d` — the pool's scheduler counters (`epoch`, `sleepers`, `far`, `released`, `threads`,
  each source's `running`, and `quiet` where this tree has it) are wrapped in `Padded`, a
  `#[repr(align(64))]` cell, so no two share a cache line. Adapted to this tree's `Entry`, which has
  no `last_job`/`quiet` fields: only `running` is padded. The `padded_counters_bench` example is
  included.
- `15824fc6f3` — a structure reference check tests grid candidates by their grid position without
  building the candidate list, and the busy-window check collects references in two passes
  (`deferred`) so a busy neighbour no longer forces the slow path. This needed the prerequisite
  fix `24dcd97129` (`dimension_origin` placement, ported just below) because the range check adds a
  `Placement::DimensionOrigin` arm. The upstream `references_preserves_grid_order_across_busy_and_
  deferred_chunks` test targets a newer `StartContext` (`base_columns_by`/`noise_column_into`) and
  a `worldgen::structure_terrain` module this tree lacks, so it is dropped; the perf change is kept.
- `8af680316d` (partial) — `FnState` caches the `#minecraft:tick` functions resolved at a
  store load (`replace_store`/`ticking`), so the tick reads its list instead of expanding the tag
  every tick. The larger half — the `DataStore` tag-expansion rewrite (each tag resolved once,
  iterative Tarjan SCC, no recursion) — targets a newer pre-image with `invalid_function_tags`
  and `remove` tag entries, absent here, so it is left out. Adapted to this tree's
  `game/functions.rs` and its 2-tuple `function_tags`.
- `616c4fec2c` — `ChunkEncoder` gains `retain_wire`: with one receiver (solo) encoded wire
  sections and heightmaps are not kept across sends, `set_retain_wire` drops what is stored and
  `clear_batch_wire` runs after a batch is encoded; multiplayer keeps the cache. Adapted to this
  tree's per-column `body_with` (upstream's `bodies_with` batch is absent) and its bulk
  block-change path, which also warms, so it sets the flag and clears afterwards too. Upstream's
  hub test uses an absent `cache_census` and is dropped; the `chunks.rs` test is ported.
- `533bb2cdd3` — bit-exact noise fast paths. `SimplexNoise` keeps a doubled permutation table
  and a `perm % 12` table (no inner `& 255`, no remainder). `PerlinNoise` keeps `perm` (257
  entries, index 256 mirrors 0), `perm_grad` (`perm & 15`) and, for `y == 0`, precomputed X-to-Y
  permutation tables plus the gradient's Y term times `dy`, so `get2` and the `y == 0` `get3`
  sample one fixed cell; `cell_grads`/`sample_fast` replace four `permute` calls per corner.
  `NoiseStack` downcasts homogeneous layers through `Any` and shares the same `Arc`s, so
  perlin/smeared/simplex stacks call the concrete methods instead of `Arc<dyn Noise>`; mixed or
  wrapped stacks keep the dynamic path. Adapted to this tree: it has no
  `add_to_needed`/`needed` mask, so the fast-layer volume loop keeps `add_to_volume` only, and
  `get2_f64`/`get3_f64` are added where this tree lacked them. The pre-change scalar
  implementations are kept under `cfg(test) mod reference` and compared bit-for-bit across the
  256 wrap, `y == 0`, `-0.0` and Far-Lands coordinates; this tree's vanilla seed and
  batched-volume tests still pass. `GradientTable::permute`/`gradient` are removed (unused).
- `ad406095d4` (partial) — the particle tick half: `tick()` skips taking and re-allocating the
  emitter, particle and `pending` buffers when they are empty, `tick_starters` retains starters in
  place with `retain_mut` instead of re-pushing every live one, and the `pending` buffer is kept
  warm across ticks (shrunk only after a >4096 bulge). `make` resolves a particle's sprite-set
  index once and hands it to `sprite_from_set_index`, so the item, vibration, firefly and default
  spawn paths no longer redo the set lookup; `fluid_kind`/`fluid_of`/`collision_top_of` let the
  bubble, bubble-column, underwater and water-drop behaviours read one block state instead of
  re-fetching it through three accessors, and the geyser, gust and explosion-seed emitters build
  their `ParticleOptions` once per burst. Dropped: the render-side zero-alloc billboard /
  dense-light-cache rewrite, which needs particle groups, the additive vertex buffer, occlusion
  culling and polytone stores this tree lacks (`particles.rs` diverges by ~1500 lines); porting it
  blind would regress more than it saves.
- `e08d2e53a5` — coalesced OIT passes. `Oit::run` now draws a frame's translucent features
  (text, crumbling, swirl, shadow, particles, items, entities, additive, weather, lightning, border)
  inside the terrain pass when no water mask patched the terrain depth (`FeaturesPass::Coalesced`;
  with a mask it keeps the separate features pass, `FeaturesPass::Separate`, and with no feature
  draw at all both are skipped, `Skip`). That removes a full-screen store and load per OIT stage.
  `gfx.rs` builds an `oit::FeatureCounts` to decide `Skip`, so a frame with nothing in that stage
  no longer runs the pass; it counts `translucent_crumbling_quads` too (the old inline check
  counted only `crumbling_quads`) and requires a visible border side. The Haar wavelet
  accumulators in `oit.wgsl` are unrolled over the three constant levels (no `exp2`, no
  dynamically indexed local array), with the coefficient lane read by `oit_lane`; a Rust
  reference port of the old loop is compared bit-for-bit in the tests. Adapted to this tree: the
  upstream `pixel_count`/`pack_timing::counted` per-stage timing wrapper is absent here, so the
  coalescing is done without it, and the shader-parsing test that used `gfx::terrain_shader()` is
  dropped (the three shader-validation tests still cover the WGSL).
- `51670062e1` — flat `SplineSampler` arena. `CubicSpline` compiles once into a
  `CompiledSpline` arena (nodes, locations, derivatives, children as `Box<[..]>`, children as
  `CompiledChild::Constant`/`Node`), and a sample walks node indices instead of a recursive
  `Box`/`Vec` tree, so the scalar path is 1.61x faster and a volume pass no longer allocates
  per cell. Coordinate samplers are filled lazily into recycled `Vec` buffers in coordinate-ID
  order (`BufferSlots`, inline when there are at most eight), and `sample_needed` evaluates only
  the masked Ys. The arena path has bit-exact tests against a recursive reference (depth 4,
  NaN/unsorted/duplicate/wide points, coordinate overflow above the inline limit). Adapted to
  this tree: upstream's `pointwise_inputs`/`is_pointwise` lattice-reuse helpers are absent here,
  so only the `sample_needed`/`sample_with`/`usable_mask` pieces the spline needs were added to
  `density/mod.rs` (default `pointwise_inputs` returns `None`, the spline reports its
  coordinates); `needed_where` followed with the `12cd1ed563` port, and the test that asserts the
  spline is pointwise keeps working through that split.
- `a8e58dcedf` (partial) — decoration plan and placement scratch reuse. `FeatureDecorationPlan`
  pre-resolves each biome/step list to step-local feature indices (`biome_steps`, an
  `FxHashMap`) instead of resolving on every chunk, and the 3x3 biome scan deduplicates ids in a
  fixed 128-word bitset with a `BTreeSet` spill, pushing each present biome's step lists directly.
  A step with at most 512 features selects its draws in eight stack words rather than a
  `BTreeSet`; `FeaturePlacer`'s single-modifier path collects positions in a 16-entry stack array
  and spills to `modified` only past that. `BiomeTemperature`'s cache uses `FxHashMap`, and
  `OreFeature` keeps its vein data and tested bitset in stack arrays up to 64 entries. The
  commit's stand-in chunk pooling (`generator.rs`, `generator_decoration.rs`) is not portable:
  it needs the `ImposterProtoChunk`/`is_full_chunk` stand-in refactor, which this tree predates.
- `0c2cff4f46` (partial) — `PageRuns` sorts a frame's runs with an O(N) counting sort over
  `(group, band, page)` buckets (up to 4096 buckets; stable, so runs stay nearest first), with
  the stable `sort_by_key` kept for sparse synthetic inputs. `PageRuns::push_in` tracks the
  group/page mix and maxima it needs (`mixed_gp`, `max_group`, `max_band`, `max_page`,
  `total_draws`), `band_of` caches the last band's squared-distance bit range so a run's band
  skips the `sqrt` division when it lands in the same 64-block band, and `regroup` copies the
  draws/pages into reused scratch vectors instead of rebuilding them through one scratch list.
  A test compares the counting sort and the sparse fallback against the old stable key order.
  The commit's `Resident` SoA walk and L1 occlusion bitset are not portable: `gfx/resident.rs`
  and `gfx/occlusion.rs` here are the older forms (573 and 661 diff lines against the
  pre-image), so only `page_runs.rs`, which was identical to the pre-image, is ported.
- `fba8c0452c` — `HudBatch::clip_since` cuts quads where they are (kept quads move down with
  `copy_within`, a cut one writes over its own slots, only a growing cut moves the unread rest)
  through a `clip_quad` helper, instead of cloning every quad out and putting each part back.
  Adapted to this tree: `clip_quad` interpolates the rectangle's UVs and text the way this tree's
  `self.quad` path did (`clip_corner` is absent), and the `game.rs` half (a reused HUD vertex
  buffer) is already covered here by `hud_reuse`/`HudBatch::new_with`. The upstream tests are
  ported as a comparison against the old copy algorithm, minus the timing benchmark.
- `24dcd97129` (prerequisite, not in the perf list) — `dimension_origin` is decoded as
  `Placement::DimensionOrigin` and given the generator's origin chunk (`StructureRegistry::origin`
  from `NoiseSpawnFinder`/`ChunkPos::ZERO`), so a set that uses it places its start instead of
  falling through to `Unsupported`. Ported because `15824fc6f3` builds on it.

Already present in this tree (verified by their code being in `HEAD`): `ab44288662` (cloud
`CloudMask` rebuild), `f43817376a` (the whole bucketed `Uploads` staging ring with
`bucket_size`/`pick_active`/`pick_free`/`trim_idle` and `create_texture_with_data`, every
added line already in this tree's `gfx/upload.rs`, so it needs no port), and most of
`e349aeccea` (packed light BFS; only upstream's later `Light::from_slice` form is absent,
this tree keeps `Light::packed`).

### Upstream performance work still to port (highest first)

Measured gains are upstream's own benchmark numbers; each needs adapting to this tree.
Every one left below is **blocked by an upstream refactor this tree does not have yet**: the
Nov 2 base predates the Nov 23-24 reorganisation, and `git apply`/hand-merge cannot bridge the
gap without porting the intermediate commit first. The prerequisite is named for each.

| Commit | Gain | What | Blocked by |
| --- | --- | --- | --- |
| `ba54be7849` | +7.6..8.9% RD16 | `glyph_scratch`, inline `DrawnKey` for the HUD | needs the `GuiScale`/`window::GuiSize` scale refactor, `HudBatch::new(window, GuiScale)`, `HudBatch::scratch`, the `pictures` field, `glint_colour`/`pvp_glint_texel`, `inline_profiles: Vec<Arc<Value>>`, `GuiDraw::FillGradient` and `hud_layout_gradient_tests.rs` — all later upstream commits |
| `eee912b040` | +4.3..8.4% RD16 | spatial front-to-back meshlet clustering in the mesher | modifies `mesher/meshlet.rs`, absent here (mesher is the older 2-file form) |
| `e549bffef3` | +3.6..8.9% RD16 | reverse-Z sky pass after solid terrain | modifies `gfx/celestial.rs`, absent here |
| `432d764acb` | +22.6..37.7% stress BE | single-pass block-entity prep, pre-baked shulkers | modifies `game/blocks/banners.rs`, `game/blocks/block_entity_anims.rs`, `game/entities/minecarts.rs` — this tree keeps `game.rs` monolithic |
| `9ba83fa92f` | +144% stress items | static item-model resolution cache, zero-alloc item prep | needs the `BakedEntry`/`LayerContent`/`prepare_parts` item-render refactor, absent here |
| `5cbec47ee2` | +4.2% stress mobs | per-frame entity/mob/item render-prep allocations | modifies 10 `game/entities/*` and `game/interface/*` files this tree does not have |
| `cbac118119` | 92x | `PendingUploads` priority queue, batched `Job::Free` | the queue lives in `horizon-client-gameplay/`, a crate that does not exist here; the integration sites are `game/interface/*` and `game/world/world_update.rs` |
| `8c27fbd2a0` | +2.0..3.8% RD32 | O(1) calm-frame LOD uploads, COW dispatch snapshots | modifies `gfx/world_handover.rs`, absent here; 1405 lines against a diverged `lod.rs`/`mesher.rs` |
| `e8618f82ba` | +6.0..10.0% | `ChunkMap` toroidal cache kept hot | the toroidal grid came from an earlier absent commit; `chunk_map.rs` here is 385 lines against 1165 pre-image |
| `d8f558197d` | +8.7..11.6% RD32 | conservative reversed-Z Hi-Z pyramid occlusion culling | adds `gfx/gpu_culling.rs` and `gfx/gpu_occlusion.wgsl`, absent here |
| `ebb6ee7a8a` | server | share encoded chunk bodies | needs `Body::Shared`/`Encoder::bodies_with`, absent here: this tree's `Body` is `Level(usize)`/`Saved(Arc<[u8]>)` and `send_many` takes owned `Vec<u8>`, so the sharing would need the owned-vs-shared send body first |
| `3fa98d1bd8`, `f71ddcce2d` | font | large multi-region rewrites | diverge from this tree's older forms, deferred |
| `dedb0375a4` | sprites | streaming sprite/CTM layout | `sprites.rs` here is 1319 lines against a 2349-line pre-image; 23 of its 25 hunks reject, so it needs the absent png/animation-layout refactor first |
| `60c8f9d0ef` | audio | single-pass mixer | `audio.rs` here is 1916 lines against a 3575-line pre-image and `sound_engine.rs` 1638 against a larger one; 42 hunks reject (30 `audio.rs`, 12 `sound_engine.rs`), so the newer sound-engine form has to land first |
| `211e07fc54` | worldgen | bounded surface-rule cache, fast column writes | 7 files and 1908 insertions against this tree's 563-line `rule.rs` and 442-line `material_stage.rs`; 24 hunks reject, so it needs the absent material-condition refactor first |
| `e0c1233a27` | -22..-33 MB | zero-alloc `EntityBatch` LUT + Schwartzian sort | needs the 16-material/native-terrain overlay refactor; this tree's `entity_batch.rs` is the older 10-material, 400-line form |
| `d939fd378d` | +2.5..3.3% RD | faster `ChunkMap` lookups | needs the toroidal-grid/hash refactor in `world.rs`/`chunk_map.rs`, absent here |
| `8e0b8ff5a5` | +1.1..4.3% RD16 | caching lightmap and skipping unchanged per-frame GPU uploads | ported in full: lightmap `LightmapKey`, the hand/crosshair global-upload skip, the sunrise-fan skip and `AnimatedTickCache` (see the list above). Adaptation: no `has_modes`/HUD-sprite skip (no Polytone `mode`s here, and the HUD sprite list has no producer) |
| `cb2132352e`, `0c2cff4f46` | +FPS / +SoA | `ColumnSlab`, SoA `Resident`/pages | `0c2cff4f46`'s `PageRuns` half is ported (see above); the `Resident`/`occlusion` SoA rewrites and `cb2132352e`'s `ColumnSlab` still diverge heavily from this tree's older `gfx/occlusion.rs`, `gfx/pages.rs`, `gfx/resident.rs` |
| `0f0dea6114`, `96d758bf25` | +1.4..2.0% RD / mesh lock | streaming mesh-job pooling | `0f0dea6114` needs `mesh_settings()`/`submit_mesh_candidates` and `hz_laps`, absent here; `96d758bf25` fixes a queue lock held across job building, which this tree never does (`submit` takes the lock per job and builds each region before it) |
| `5aeaafa6e5` | world | remap section palette slots at pack time | this tree's `PalettedContainer::to_nbt` predates the per-cell dedup the `aliases` table optimises (it copies the palette and words as-is), so the change would replace a cheaper save with a remap |
| `a3ae5b0d11` | hub | meet a tracked-whole cell without its entities | needs the multi-level viewer (`Viewer::sources`, `player_shown`, `viewed_levels::entities`) and the `lan::Body` output this tree's tracker predates |
| `42af125dc8` | render | GPU skinning for unarmored players / tagged translucent bodies | needs `paint` in the `posable` check and the `creatures/mod.rs` form; this tree's `posable` has no tint path there |
| `1181594273` | vehicles | slot test before `passenger_object_present` | this tree has no `persist.passenger_slots` (older passenger graph) |
| `918ae6d651` | commands | selectors read the player list alone | this tree's `Selector` has no `include_entities` field (later selector refactor) |
| `1dc5634a22` | hub | route `par_chunks_mut` to the persistent pool | this tree has no `parallel.rs` / `horizon_threads::par_chunks_mut`; the tick helpers are a later addition |
| `a8e58dcedf` | -15.1% decorate | stand-in chunk pooling, ore/placement scratch | decoration plan/placement scratch/biome-cache parts ported (see above); the stand-in chunk pooling itself still needs the `ImposterProtoChunk`/`is_full_chunk` refactor and `TargetMemo`/`SphereSquares`, absent here |

Ported and listed above: `326d390035`, `0a4fe8c212`, `5c5e6b1651`, `b17d429e33`, `5db9bc3f45`,
`d41da14ca2`, `1fd5f925a8`, `55aa15efeb`, `6dbd04fc1a`, `64baac04dd`, `3c678f8923`, `8060e1ff24`,
`b6562f83a8`, `bc6bc483a8`, `7cc881c0e3`, `19c9350647`, `576b17b8a9`, `7f693e21b4`, `746f280e8e`,
`150a1013d2`, `635586ce1b`, `a03f108dbc`, `12cd1ed563`, `46e0a6942e`,
`c415b158fe`, `92abe36aec`, `4ac255c284`, `fdf87ca16d`, `ec0648aba7`, `debef4584c`,
`07147ef4af`, `1bde596b8d`, `fd3b4ff6a3`, `9609bc8ccf`, `d9b8e7bef2` (compact table half),
`e0bbdc1e25`, `92d8bf33e1`, `8197ba2363`, `dd3ed413f2`, `15824fc6f3` (with prerequisite
`24dcd97129`), `8af680316d` (partial), `616c4fec2c`, `fba8c0452c`, `533bb2cdd3`, `ad406095d4`
(partial), `e08d2e53a5`, `51670062e1`, `12cd1ed563`, `a8e58dcedf` (partial), `0c2cff4f46` (partial).

### Every remaining upstream perf commit and why it is deferred

The commits below are the rest of the Nov 20-24 window's performance batches. `Kind` says why
each is left out: **files absent** / **partially absent** touch files this tree never had (the
Nov 23-24 gameplay/render reorganisation), so there is nothing to apply them to; **api/semantics**
has every named file here, but the change calls APIs or depends on data this tree's older form
does not have. The notable `api/semantics` cases are detailed in the table above; the list here is
the complete accounting. (`ab44288662` and `e349aeccea` are already present in `HEAD`, and the two
`docs(benchmark)` commits and `c2ee7380c3` (shader packs, removed here) are not applicable.)

| Commit | Kind | Files | Absent | Subject |
| --- | --- | --- | --- | --- |
| `3a0c68c1c9` | api/semantics | 1 | 0 | docs(benchmark): the A/B example passes horizon-perf summary the newest report (#19753) |
| `3465b5f7ac` | api/semantics | 1 | 0 | docs(benchmark): the A/B section reads its result with horizon-perf summary (#19745) |
| `60c8f9d0ef` | api/semantics | 3 | 0 | perf(audio): 13.15x faster mix_buffer (-92.4%) and 4.86x faster SoundEngine::tick (-79.4%) via single-pass mixer (#19535) |
| `96d758bf25` | api/semantics | 1 | 0 | perf(chunks): build mesh jobs before taking the job queue's lock (#19870) |
| `f71ddcce2d` | api/semantics | 1 | 0 | perf(font): LZ4-compressed chunked off-atlas bitmap font strips (#20186) |
| `3fa98d1bd8` | api/semantics | 1 | 0 | perf(font): compact Unihex HexTable and shared archive cache (~7.4 MB less RAM per table) (#20185) |
| `8e0b8ff5a5` | api/semantics | 2 | 0 | perf(render): +1.1% to +4.3% FPS (+9.5% 1% lows) by caching lightmap and skipping unchanged per-frame GPU uploads (#19513) |
| `0f0dea6114` | api/semantics | 1 | 0 | perf(render): +1.4% to +2.0% FPS and -56 MB mountain RSS via section-level mesh budgeting and calm-frame guards (#19527) |
| `d939fd378d` | api/semantics | 2 | 0 | perf(render): +2.5% to +3.3% FPS and 1.59x faster lookups via safe toroidal ChunkMap cache (#19515) |
| `f43817376a` | api/semantics | 1 | 0 | perf(render): -13 to -25 MB steady RSS and +0.9% FPS via bucketed Uploads staging ring with idle trimming (#19516) |
| `e0c1233a27` | api/semantics | 1 | 0 | perf(render): -22 to -33 MB steady RSS and zero-alloc EntityBatch LUT + Schwartzian sort (#19520) |
| `dedb0375a4` | api/semantics | 1 | 0 | perf(render): -8.2% to -8.6% sprite and CTM pack load time and lower peak RAM via streaming layout (#19534) |
| `cb2132352e` | api/semantics | 1 | 0 | perf(render): contiguous ColumnSlab and zero-alloc scratch in cave occlusion graph (-5.0% RD 34 search_cost) (#20471) |
| `ebb6ee7a8a` | api/semantics | 1 | 0 | perf(server): share encoded chunk bodies across recipients (#18368) |
| `c2ee7380c3` | api/semantics | 1 | 0 | perf(shaders): a shader pack's global of literals is initialized by the module, not stored again in every invocation (#19715) |
| `211e07fc54` | api/semantics | 7 | 0 | perf(worldgen): bounded per-worker CompiledSurfaceRule variant cache and fast column writes (#20190) |
| `f618858113` | files absent | 4 | 4 | bug(mods): a game with native mods keeps Play with Friends and Horizon's other official parts (#19408) |
| `937363f340` | files absent | 3 | 3 | fix(tooling): reading a .hzperf capture stops at a limit on what it inflates (#20350) |
| `bf1294c568` | files absent | 7 | 7 | perf(carts): a cart takes the boxes of the carts near it once when its tick begins (#18043) |
| `f82d68ab08` | files absent | 2 | 2 | perf(client): -48 MB peak RSS and per-item Remote backlog deadline preventing chunk-burst starvation (#19529) |
| `918ae6d651` | files absent | 1 | 1 | perf(commands): player selectors read the player list, not every entity of every level (#19242) |
| `0708a190c6` | files absent | 4 | 4 | perf(commands): use an ordered TimerQueue for scheduled functions (#20405) |
| `d1327bf82c` | files absent | 2 | 2 | perf(game): index player bot movement lookup (#19752) |
| `df587802f0` | files absent | 1 | 1 | perf(json-nbt): avoid per-number string and per-object Vec allocations in json_nbt (#20643) |
| `210fdc3e6c` | files absent | 3 | 3 | perf(lan): add O(1) GuestList UUID index, lock-free layout/written check, and registration_ref (#19082) |
| `4b4a6b40e1` | files absent | 2 | 2 | perf(mobs): serialise a mob's turn only for the hooks that run in it (#18260) |
| `a87cdbd698` | files absent | 15 | 15 | perf(mod-api): keep per-file generator results across build script runs (#18249) |
| `4529f78913` | files absent | 1 | 1 | perf(mod-runtime): skip byte_channel condvar wakeups when no waiters exist (#19084) |
| `40879f5f25` | files absent | 5 | 5 | perf(modding): track mob/player render hook listeners per phase via MobRenderHooks (#19580) |
| `3ff2a22128` | files absent | 1 | 1 | perf(mods): fluid declarations load through the shared package reader (#20067) |
| `4291720f70` | files absent | 2 | 2 | perf(motion): add cells_without_collision fast path for no_player_collision (#19550) |
| `ff7679e3eb` | files absent | 3 | 3 | perf(portals): each level keeps its own portal palette cache (#18365) |
| `ac8c53a1e7` | files absent | 3 | 3 | perf(render): +1.6% to +4.8% FPS (+15.5% 1% lows) via 4-band coalesced GPU culling (#19509) |
| `73c70814c7` | files absent | 1 | 1 | perf(rendering): equal LabPBR map pages share one page of the paged map's pool (#18918) |
| `9436a2972a` | files absent | 3 | 3 | perf(rendering): the frame check reads nothing from disk with no shader pack picked (#19216) (#19254) |
| `f8143deba2` | files absent | 3 | 3 | perf(scoreboard): find the teams of entities by the holders on teams, not by formatting every entity's name (#18017) |
| `a95ec735d0` | files absent | 3 | 3 | perf(server): a guest's view for the mobs swaps the dozen fields it reads, not a hundred (#18033) |
| `b3238d7e7a` | files absent | 4 | 4 | perf(server): an idle level closes without waiting for the disk in the tick (#18474) |
| `3211b1c783` | files absent | 4 | 4 | perf(server): refresh the guest turns' player boxes by write, not for every guest (#17384) |
| `6ab1ecf4df` | files absent | 1 | 1 | perf(server): the proxy keeps the account service's discovery answer for five minutes (#20133) |
| `9369bd7696` | files absent | 1 | 1 | perf(snbt): add ASCII fast paths and avoid intermediate allocations in SNBT parser (#20641) |
| `fc2be616c9` | files absent | 12 | 12 | perf(stats): store per-tick custom counters inline and privatize Stats::values (#19323) |
| `1181594273` | files absent | 1 | 1 | perf(vehicles): find a rider's vehicle by its slots before asking the vehicle's presence (#18377) |
| `82a7d7af03` | files absent | 2 | 2 | perf(world): skip random ticks and empty block_events with_level work in tick_blocks (#19454) |
| `406fc22363` | partially absent | 2 | 1 | build(perf): stop building horizon-perf as cdylib by default (#20515) |
| `3b1d03eb67` | partially absent | 8 | 6 | fix+perf(gpu): fix live GRAIN=64 shift in mesh/visbuffer shaders, add workgroup compaction, staged uploads, and cached visbuffer bind groups (#20203) |
| `938b5387c4` | partially absent | 4 | 3 | perf(advancements): pre-filter tick criteria and cache biome/structure lookups (#19551) |
| `e8bbb95d45` | partially absent | 5 | 4 | perf(advancements): the location trigger does not ask again what it found unmet for the same biome (#19863) |
| `9bda820d2f` | partially absent | 5 | 4 | perf(blocks): borrow crafting grids and reuse non-empty stack slices per recipe lookup (#20456) |
| `5cbec47ee2` | partially absent | 25 | 12 | perf(client): +4.2% stress_mobs FPS and lower per-frame entity/mob/item render-prep allocations (#19531) |
| `e095541dc6` | partially absent | 2 | 1 | perf(client): -35 MB RD 32 steady RSS via reusable remote chunk DecodeScratch and palette-safe uniform pooling (#19530) |
| `1d0b68bcbb` | partially absent | 10 | 6 | perf(client): Battery Mode halves every pool the client starts (#18465) |
| `35182ca465` | partially absent | 6 | 1 | perf(creative): index stack deduplication during tab rebuilds (#18459) |
| `c94d887bda` | partially absent | 22 | 18 | perf(entities): an armor stand nobody has borrowed mutably keeps its network entry (#19864) |
| `f60fd796db` | partially absent | 6 | 4 | perf(far-terrain): limit far work to one job at a time through a world entry's first near load (#20457) |
| `ee65f0d916` | partially absent | 3 | 2 | perf(far-terrain): summarize a near chunk when it leaves the near area, not when it arrives (#20455) |
| `755f7892ac` | partially absent | 6 | 5 | perf(game): the server's tick stops doing client work (#15437 Wave 0) (#18076) |
| `d8f558197d` | partially absent | 6 | 2 | perf(gpu): +8.7% to +11.6% RD 32 FPS (+18.5% mountain 1% low) via conservative reversed-Z Hi-Z pyramid compute occlusion culling (#20194) |
| `150cf23815` | partially absent | 4 | 1 | perf(graphics): pbrBudgetMiB sets the LabPBR maps' video memory apart from the block textures' (#18690) |
| `a3ae5b0d11` | partially absent | 3 | 2 | perf(hub): meet a cell the client tracks whole without its entities (#18041) |
| `1dc5634a22` | partially absent | 3 | 1 | perf(hub): route remaining par_chunks_mut callers to the persistent tick-helper pool (#19074) |
| `ba54be7849` | partially absent | 5 | 1 | perf(hud): +7.6% to +8.9% RD 16 FPS (+12.2% to +16.9% 1% low) via reusable HudBatch/GuiDraw/glyph_scratch and inline DrawnKey (#20196) |
| `9ba83fa92f` | partially absent | 6 | 3 | perf(items): +144.4% stress_items FPS (+50.9% stress_frames) via static item model resolution cache and zero-alloc item prep (#20192) |
| `e0fa72f5db` | partially absent | 6 | 1 | perf(launcher): adopt async engine for Minecraft downloads (#15970) |
| `c731ef64c2` | partially absent | 4 | 2 | perf(locator-bar): a calm tick meets only what changed; flat pair step; delivery by receiver (#17446) |
| `17b92d07c2` | partially absent | 4 | 2 | perf(locator-bar): encode a waypoint update once for all its receivers (#17382) |
| `8c27fbd2a0` | partially absent | 5 | 1 | perf(lod): +2.0% to +3.8% RD 32 FPS via O(1) calm-frame LOD uploads, COW dispatch snapshots, and skipping unused LOD meshlets (#20200) |
| `eee912b040` | partially absent | 3 | 1 | perf(mesher): +4.3% to +8.4% RD 16 FPS (+11.9% to +19.5% 1% low) via spatial front-to-back meshlet clustering and branchless sub-AABB (#20197) |
| `e2f71bb180` | partially absent | 4 | 1 | perf(mobs): add static mannequin fast path with world revision and position stamp (#19455) |
| `eb230e5a2b` | partially absent | 15 | 9 | perf(mobs): keep the villagers' village sections in an index for the mob loop (#19843) |
| `b6e61677b7` | partially absent | 4 | 2 | perf(mobs): lock-free seqlock PathTypeCache (4.44x faster under 8-thread contention) and zero-alloc A* Finder (#20477) |
| `a82b6173e5` | partially absent | 18 | 11 | perf(mobs): search a target among the slots of its species, not every mob nearby (#18029) |
| `8989a510ff` | partially absent | 7 | 5 | perf(mods): link a content root's directories as junctions on Windows (#17546) |
| `37352802a1` | partially absent | 3 | 1 | perf(motion): add stationary fast paths for replayed_tick, server_travel, and flat support (#19072) |
| `7954f21558` | partially absent | 4 | 2 | perf(network): frame the small packets of a run in place, counted once (#17380) |
| `bfa705b62a` | partially absent | 6 | 5 | perf(protocol): scope legacy guards to their actual readers (#20212) |
| `a71b87a3d5` | partially absent | 3 | 2 | perf(protocol): serialize network NBT directly into packets (#19931) |
| `82e61665c4` | partially absent | 5 | 2 | perf(render): +17.3% ocean and +8.7% village chunk meshing throughput (-25.4% max section) via mesher fast paths (#19523) |
| `231984a0df` | partially absent | 11 | 6 | perf(render): +2.0% to +4.6% FPS (+14.0% 1% lows) by staging TAA/OIT/HDR uniforms and gating unhooked mod slots (#19511) |
| `432d764acb` | partially absent | 11 | 4 | perf(render): +22.6% to +37.7% stress BE FPS (+2.9% village) via single-pass block-entity prep, pre-baked shulkers, and const diffuse lights (#20193) |
| `e549bffef3` | partially absent | 5 | 2 | perf(render): +3.6% to +8.9% RD 16 FPS (+7.0% to +20.2% 1% low) on Linux Vulkan via reverse-Z sky pass after solid terrain (#19510) |
| `916563f8ce` | partially absent | 4 | 2 | perf(render): -37 MB mountain RSS and +1.6% RD 32 FPS (~55% smaller LOD tiles) via flat-CSR TileColumns (#19521) |
| `5fc0fdfdb9` | partially absent | 8 | 6 | perf(render): -71 to -144 MB VRAM and up to +2.2% FPS (+10.0% 1% lows) via Arena GRAIN=64 and O(1) FreeList (#19512) |
| `fa67d4f767` | partially absent | 6 | 2 | perf(render): GPU Culling Auto keeps a trial only when frame time holds (#19972) |
| `45eeccded1` | partially absent | 7 | 4 | perf(render): GPU compute meshlet culling and Visbuffer mesh shader support (#19485) |
| `42af125dc8` | partially absent | 2 | 1 | perf(render): enable GPU skinning for unarmored remote players and translucent-body mobs when tagged (#19581) |
| `b45d73de43` | partially absent | 2 | 1 | perf(render): skip off-screen weather column generation and vertex upload (#15997) |
| `d31d4b41d0` | partially absent | 4 | 3 | perf(rendering): a depth pre-pass for a native pack's solid material terrain (#20251) |
| `b3e502a5e2` | partially absent | 2 | 1 | perf(rendering): bake a server pack's own item models ahead, on a worker (#18177) |
| `dc6202f313` | partially absent | 5 | 3 | perf(rendering): hardware task and mesh shader terrain stages (step 2) (#17891) |
| `c3b758468c` | partially absent | 7 | 5 | perf(rendering): section meshlet extraction, GPU store and mesh pipeline (step 3 & 4) (#18517) |
| `917f45a488` | partially absent | 6 | 4 | perf(rendering): upload solid_sources by slot for packed flat shader packs (#19605) |
| `1e1e41d912` | partially absent | 2 | 1 | perf(rendering): with HDR on, the display check no longer stalls the render thread every 2 s (#18609) |
| `a27fee23ca` | partially absent | 7 | 5 | perf(rendering): worker-thread meshlets, occlusion culling, indirect dispatch and 128-lane mesh shader (#19471) |
| `a05c0e5aa0` | partially absent | 5 | 3 | perf(saves): lazily cache canonical section palettes per SaveDir (#19440) |
| `309b772964` | partially absent | 5 | 3 | perf(saves): share one section palette budget per world and count shared entries once (#19861) |
| `5b21a55844` | partially absent | 10 | 4 | perf(server): give memory back once a dedicated server is empty (#18290) |
| `5115507c83` | partially absent | 12 | 6 | perf(server): give the server thread a physical core of its own on machines with SMT (#17840) |
| `7fe978ddbe` | partially absent | 5 | 4 | perf(server): run a guest's sculk step and tick tail in its body's turn when it told no one anything (#18031) |
| `55cc2b9740` | partially absent | 6 | 5 | perf(server): share identical block and light broadcast payloads (#20238) |
| `2798431890` | partially absent | 8 | 3 | perf(shaders): a pack's shadow pass sorts its sections once and draws the solid ones with indirect multi-draws (#19806) |
| `6fd61f2dcc` | partially absent | 19 | 15 | perf(shaders): compile native shader packs asynchronously and batch option edits (#20601) |
| `cbac118119` | partially absent | 9 | 6 | perf(streaming): 92x faster PendingUploads priority queue, O(1) settlement checks and batched Job::Free lock (#20473) |
| `e8618f82ba` | partially absent | 13 | 11 | perf(world): +6.0% to +10.0% FPS (+19.9% village 1% low) by keeping ChunkMap toroidal cache hot via grid_missing counter and pub mutate_chunk (#20195) |
| `182e08c079` | partially absent | 4 | 1 | perf(world): add O(1) id index and well-known lookup to LiveAttributes (#19077) |
| `276a8e844f` | partially absent | 4 | 2 | perf(world): cache awake block entity tickers and refresh mid-pass on wake (#19079) |
| `2b13c06240` | partially absent | 6 | 3 | perf(world): far terrain stays below normal priority on Windows; no auto-pause during loading (#19957) |
| `da428edd27` | partially absent | 12 | 11 | perf(world): index block entities by kind and by chunk (#18001) |
| `5aeaafa6e5` | partially absent | 3 | 1 | perf(world): remap section palette slots at pack time (#20095) |
| `839c063c9e` | partially absent | 9 | 3 | perf(worldgen): -4.4% proto wall (-22.9% biomes) at 6,400 chunks via 64-shard caches, shared Arc<BiomeChunk>, and ClimateIndex (#19537) |
| `fa3e009465` | partially absent | 17 | 14 | tooling(perf): add headless baselines for five engine hot paths (#19968) |

Deferred: 136 commits — 25 api/semantics, 79 partially absent, 32 files absent.
