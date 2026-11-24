<p align="center">
  <img src="https://img.shields.io/badge/Minecraft-Java%2026.3-5D9C3F" alt="Minecraft Java 26.3">
  <img src="https://img.shields.io/badge/Rust-stable-B7410E?logo=rust&logoColor=white" alt="Rust">
  <a href="https://github.com/ludovic111/horizon"><img src="https://img.shields.io/badge/fork%20of-Horizon-F0795A" alt="Fork of Horizon"></a>
  <img src="https://img.shields.io/badge/platform-Android-2A2470" alt="Android">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-3B2F7E" alt="MIT license"></a>
</p>

<p align="center">
  <a href="#download"><b>Download</b></a> ·
  <a href="#what-tml-is"><b>What TML is</b></a> ·
  <a href="#performance"><b>Performance</b></a> ·
  <a href="#files-and-storage"><b>Files</b></a> ·
  <a href="#build-from-source"><b>Build from source</b></a>
</p>

# TML Android

**TML is a lightweight, smoothness-first port of Minecraft: Java Edition 26.3 to Android.** It is a
fork of the Horizon rewrite of the Java game (via Starling / Horizon Android): it reads the same
worlds, follows the same rules and generates the same terrain as Java Edition, and runs natively —
no Java virtual machine, no OpenGL translation — so it starts fast and stays smooth on a phone.

> Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft. TML
> contains no Minecraft code, assets or data: you need your own legally obtained copy of Minecraft:
> Java Edition 26.3.

## Download

Grab the newest `tml-<version>-arm64-v8a.apk` from the
[releases page](https://github.com/TimeAndTimeStudio/TML-Android/releases) and open it on the phone
(Android asks you to allow installing apps from your browser or file manager).

**You need** Android 8 or newer, a compatible graphics driver, and about 2 GB free. Sign in with a
Microsoft account that owns Minecraft: Java Edition, or play offline (most servers need the
sign-in). No computer: on the first start TML downloads the game's files from Mojang's servers and
prepares them on the phone, once, with full per-step progress.

## What TML is

- **The game:** singleplayer with vanilla's world generation, mobs, villagers, raids, redstone,
  every command, the HUD and the recipe book, on saves in vanilla's format. Worlds from 1.8 to 26.2
  are upgraded as vanilla does. Multiplayer, LAN and servers work as upstream does.
- **Setting up on the phone:** the game's files are downloaded and prepared on the phone by a Java
  runtime fetched just for that and removed after. Preparation reports every step with file counts;
  there is no "import game files" button and no all-files-access permission — the game's files live
  in the app's own storage.
- **Touch controls:** a movable, resizable, fadeable layout; a stick, a D-pad or Bedrock-style tap
  to interact; gyroscope aiming off by default; haptics on block breaks.
- **Resource packs:** resource packs and server resource packs load; a server's pack **asks first**
  (vanilla's prompt stays the default — nothing is accepted automatically).
- **Bundled TML UI pack:** a new install gets TML UI once; remove it and it stays removed.
- **Your files:** worlds, resource packs and options live in a `TML` folder on the phone's storage,
  openable by any file manager. The big game files (`resources/`, `minecraft/`) stay in app storage
  and are never copied to shared storage.
- **Diagnostics:** FPS, memory and temperature overlay, crash reports and an exportable diagnostic
  ZIP under **Options → Diagnostics**.

## No shader packs

TML ships **without a shader pack system** — no Iris/OptiFine `shaders/` packs, no bundled shader
pack, no shader options screens. The game draws with its own pipeline only, which is what makes it
smooth on mid-range Mali and Adreno phones: fewer passes, no per-pack translation, no pack-specific
crashes or driver stalls. Shaders are simply Off, always.

What TML keeps from the rendering work: GPU occlusion culling of terrain sections (with an
in-shader depth test on Android), indirect-draw selection measured per device, reusable HUD
buffers, skip-empty OIT feature passes, and a one-byte ambient-occlusion table in the mesher.

**Resolution Scale** (render scale) is adjustable in **Video Settings → Quality**, like every other
quality knob — nothing lowers the picture for you automatically.

## Performance

TML's whole reason to exist is frame time. The levers it ships with:

| Lever | Where |
| --- | --- |
| Occlusion culling of hidden terrain sections | on by default (`HORIZON_GPU_CULL=0` to disable) |
| Indirect multi-draws only where measured faster | auto-trialled per device, kept or dropped |
| HUD vertex/patch/blend buffers reused across frames | always |
| OIT feature pass skipped when a frame has nothing in it | always |
| One-byte AO state table in the mesher | always |
| Per-device defaults chosen on first start | Video Settings |

An `HORIZON_*` environment variable in `TML/horizon-args.txt` overrides any of the above for
testing.

## Build from source

Requirements: Rust (stable, with the `aarch64-linux-android` target), the Android SDK
(build-tools 35.0.0, platform 36, NDK r28c), a JDK and Python 3.10 or newer.

```sh
git clone https://github.com/TimeAndTimeStudio/TML-Android.git
cd TML-Android
python packaging/android/build_apk.py            # the APK lands in target/android/
```

The build compiles the exporters with the host JDK and packages their checked bundle; it uses
`HORIZON_MINECRAFT_DIR` as a read-only source when set, else downloads checksum-pinned inputs into
an ignored build cache. Minecraft jars, libraries and assets are **never** packaged in the APK — on
the phone, the game fetches them from Mojang and prepares them there.

Every push also builds an APK in GitHub Actions (`android.yml`), and successful runs on `main` are
published as releases.

## Repository layout

- `crates/` — the workspace: `horizon-client` (gameplay), `horizon-client-render` (rendering, HUD,
  screens), `horizon-world` (world data, worldgen), `horizon-protocol`, `horizon-prepare` (the
  on-phone preparation pipeline), and the rest.
- `packaging/android/` — `build_apk.py`, signing and release assets.
- `docs/` — architecture, controls, diagnostics and the port ledger.
- `SYNC.md` — the log of what has been ported from upstream Horizon.

## Licence

[MIT](LICENSE), for the project's original work only, as Horizon's. Dependencies keep their own
licences. Minecraft is a trademark of Mojang Synergies AB; Minecraft code and assets belong to
Mojang and Microsoft and are covered by the
[Minecraft EULA](https://www.minecraft.net/en-us/eula). Prepared resources stay on your devices and
are never published.

Third-party credits and licenses: [Third-party notices](THIRD_PARTY_NOTICES.md).

## Credits

The Android port lineage: Starling by [JuanLuci](https://github.com/JuanLuci) and Horizon Android
by [Vasysik](https://github.com/Vasysik). Horizon made by
[Ludovic Marie](https://lu4ovic.com) ([@ludovic111](https://github.com/ludovic111)). TML continues
that work as its own project.
