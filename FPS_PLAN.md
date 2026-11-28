# Plan: frame rate while flying and while a world loads

Status: in progress. P0 has landed (frame-time collection, upload queue); P1 is next.

## 1. The problem

Players report two slow situations on the Mali-G57 MC2 device:

| Situation | Symptom |
|---|---|
| Flying / elytra, or sprinting across new terrain | frame rate drops below 60 and stays there until the view settles |
| Entering a world, until the view is filled | frame rate is low the whole time the chunks arrive |

"Base" (vanilla) keeps above 60 in both. So the gap is in what this tree does per
frame while terrain is arriving, not in steady-state rendering (steady state was
already brought to vanilla level by the earlier ports).

Two separate causes are suspected, and the plan treats them separately:

1. **Per-frame work nobody reads.** Diagnostics, counters and sample logs that run
   every frame even though no consumer is attached (no control socket, no trace,
   no telemetry consent). This is what the report calls "logging/collecting".
2. **Pacing of terrain while it arrives.** Uploads and mesh jobs are budgeted per
   frame; when the view moves, the budget is what decides whether the frame time
   holds. This is what decides flying frame rate.

## 2. How we measure (before any change)

Every candidate is judged on the same three numbers, taken on the phone:

- **Steady-state FPS** in a settled world (baseline sanity: must not regress).
- **Flying FPS**: same flight path each time, e.g. creative flight in a straight
  line over fresh terrain for 20 s.
- **World-load FPS**: from pressing Play to the view being filled.

Instruments already in the tree:

- `FPS` in the debug overlay, plus the frame-time chart (F3+2).
- `HORIZON_SPIKES=MS` in `horizon-args.txt` → one logcat line per slow frame with
  the spans it spent its time in (`crates/horizon-client-render/src/spans.rs`).
- `HORIZON_STARTUP_TRACE=FILE` → the load timeline, per step
  (`spans::startup`, read by `tools/startup_trace.py`).
- Desktop A/B, where the control socket exists:
  `horizon-ctl fly X1 Y1 Z1 X2 Y2 Z2 SECONDS PITCH raw` returns every frame time,
  so a change can be accepted or rejected on numbers instead of impressions.

**Acceptance rule** (same as the existing culling trial in `gfx/cull.rs`): a change
ships only if the targeted situation improves by more than the run-to-run noise
(≥ 3 % on the p95 frame time over three runs), and steady-state FPS does not drop
more than 2 %. Otherwise it is reverted in the same branch.

## 3. Audit findings

### 3.1 Per-frame collection that has no consumer

| Where | What runs every frame | Cost | Verdict |
|---|---|---|---|
| `crates/horizon-client/src/game.rs:18650` → `control.rs:564` | `FrameStats::record` pushes one `f32` into a `Vec` that is allowed to grow to `1 << 20` entries (~4 MB), forever, on every platform | a push + a length check; the real cost is 4 MB of resident memory and 4 MB of growth | **gate it**: record only while a `stats`/`fly` consumer is attached, or keep a bounded ring (last ~8 k frames) |
| `crates/horizon-client/src/game.rs:18667-18672` | while `startup::filling()`, builds a `Vec` of 10 counters and calls `streaming.stages()` — which scans **every** proto chunk (`streaming.rs:1559-1571`) | one heap allocation + O(#protos) scan **per frame**, exactly during world load; `startup::counters` then throws it away because tracing is off | **move the construction inside the `startup::on()` check** |
| `crates/horizon-client/src/game.rs:16923-16939` | builds an `FxHashSet` of fresh keys and `sort_unstable_by_key`s the whole pending upload list, even when `uploads` is empty and nothing changed | allocation + O(n log n) per frame with a long pending list (a flying player keeps one) | **skip when nothing arrived**, reuse the set/sort buffers between frames |
| `crates/horizon-client/src/game.rs:16904` | `view_arrival.arriving()` walks the view ring | O(r) until the ring is complete, then O(1) | leave; it self-disables |
| `crates/horizon-client/src/game/diagnostics.rs:47` | heartbeat | `try_lock` + phase compare, writes at most every 5 s | leave |
| `crates/horizon-client/src/game/perf.rs:23` | GPU allocator walk for the overlay | 1 s interval, only while the stats overlay is drawn | leave (already opt-in) |
| `crates/horizon-client-render/src/spans.rs:164` | `spans::time` around each phase | one TLS read when off; `heap_tag` folds to nothing without the `alloc-census` feature | leave |
| `crates/horizon-client/src/game.rs:18651` | `debug_charts.frame_times.log` | bounded 240-entry ring; vanilla parity (F3 chart) | leave |

Conclusion: the unconditional collection is smaller than expected. The two real
items are `FrameStats` (memory, not FPS) and the `world.fill` counter build
(allocation + scan during load). Both are cheap to fix and risk-free.

### 3.2 Terrain pacing while the view moves

- Uploads are paced by `AUTO_UPLOAD_TIME` = 2 ms per frame, `AUTO_UPLOAD_TIME_ARRIVING`
  = 6 ms while the view is arriving, at most `AUTO_UPLOADS_AT_ONCE` = 8
  (`game.rs:471-477`). A 60 Hz frame is 16.6 ms; spending 6 ms of it on uploads
  leaves little for the rest, and the cap of 8 meshes keeps the queue long.
- Meshes in flight are capped at `workers * 32` (`streaming.rs:2016`). On a phone
  that is a few hundred small jobs competing with generation and lighting in the
  same pool.
- `Renderer::upload` (`gfx.rs:4925`) is synchronous: it writes staging buffers and
  clones block-entity and light lists per section. That is the per-upload cost the
  budget is protecting against.
- Occlusion culling is off on this device (`indirect_draws_wanted()` is false
  without `VERTEX_WRITABLE_STORAGE`), so distant sections are still drawn while
  flying; only frustum + the culling trial apply.

## 4. Work items, in order

Each item is one commit with its own version bump, its own APK, and its own
`SYNC.md` note. Items are independent; any of them can be dropped.

### P0 — stop collecting what nothing reads (no visual change, no risk)

1. `FrameStats`: bounded ring (keep the last 8 k frames) and record only while a
   control consumer is attached. Keeps `stats`/`fly` working on desktop, removes
   the unbounded growth and the per-frame push on Android.
2. `world.fill` counters: build the counter slice only when `startup::on()`.
3. Upload path: skip the `fresh` set and the sort when `uploads` is empty; keep
   the set and the sort key buffer on `Game` so a flying frame does not allocate.

Expected: small but free. Mainly removes load-time allocation churn.

### P1 — terrain pacing (the flying fix)

4. Make the arriving budget adaptive: while `arriving`, spend up to
   `AUTO_UPLOAD_TIME_ARRIVING` but stop early once the frame's CPU time already
   exceeds a target (measure with the existing frame clock, no new collection).
   This is the same "keep it only if it pays" shape as the culling trial.
5. Raise the mesh in-flight cap only when the pool is idle on generation and
   lighting (`streaming.rs:2016`), so meshes for the view arrive sooner without
   starving the chunk pipeline.
6. Prioritise mesh jobs by distance to the view direction, not only by ring, so
   what the player is flying into is meshed first.

Expected: the main flying win. Risk: overspending a frame turns into hitches, so
this is the item that most needs the A/B rule.

### P2 — render path while moving

7. Entity culling: currently `EntityCullingMode::Off` by default
   (`graphics.rs`). Turning it on for the mobile preset is a one-line default
   change with a measurable draw-count effect; check it against the acceptance rule.
8. Review the per-frame `flatten` prepare (`gfx.rs:7599`): it rebuilds instance
   data for every visible section each frame. If a section's instances did not
   change, they should not be rebuilt.

### P3 — explicitly not in this plan

- Per-section GPU occlusion culling on Mali: needs indirect multi-draw, measured
  46 ms vs 22 ms on this device. Large renderer change, negative on this hardware.
- Shader-pack, worldgen and mesher rewrites: ported or rejected already, see
  `SYNC.md`; a blind rewrite risks a worse regression than the one being fixed.

## 5. Validation protocol per commit

1. `rustfmt --check` on each changed file (this tree is not `cargo fmt` clean).
2. `cargo check -p <touched crate>` + that crate's tests.
3. `cargo check -p horizon-client --no-default-features`.
4. `python3 tools/pr_ready.py --quick`.
5. Bump `VERSION_NAME`/`VERSION_CODE` in `packaging/android/build_apk.py` and the
   fallbacks in `crates/horizon-client/src/game/update_offer.rs`.
6. `SYNC.md`: what changed, why, and what was measured.
7. Push; confirm the CI release tag and the `arm64-v8a` APK.
8. Report the before/after numbers for the situation the commit targets.

## 6. Rollback

Every item is a separate commit, so a regression is reverted by reverting one
commit. Items 4 and 5 additionally keep their old constants behind the trial
check, so they can be disabled without a revert if the numbers are ambiguous.
