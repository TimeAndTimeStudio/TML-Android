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
  inputs did not change. The sprite tick cache and per-frame upload skip are still to port.
- `8e0b8ff5a5` (part 2) — skip hand/crosshair global uploads when there is nothing to draw, and
  skip the sunrise fan upload when the sky is not visible.
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
