# paper_plane

*Leer en español: [README_ES.md](README_ES.md)*

A Windows screensaver (.scr) written in Rust: low-poly paper planes flying
along random Bézier curves over a black background.

- Multi-monitor support (an independent scene per monitor).
- HiDPI aware (Per-Monitor DPI Aware V2, rendered at native physical resolution).
- Rendered with wgpu (Direct3D 12 / Vulkan) with 4x MSAA.
- Native configuration dialog: number of planes (1–50) and speed (1–10).
- Settings are stored in `HKCU\SOFTWARE\paper_plane`
  (`PlaneCount` and `Speed`, DWORD values).

## Building

```
cargo build --release
copy target\release\paper_plane.exe paper_plane.scr
```

## Installing

1. Right-click `paper_plane.scr` → **Install**, or
2. Copy `paper_plane.scr` to `C:\Windows\System32` (or any folder) and select
   it under Settings → Personalization → Lock screen → Screen saver.

## Arguments (standard screensaver interface)

| Argument | Function |
|----------|----------|
| `/s`     | Run full screen |
| `/c` or `/c:HWND` | Configuration dialog |
| `/p HWND` | Preview inside the thumbnail of the Windows dialog |
| `/w`     | Windowed mode (extra, for debugging; press Esc to quit) |
| *(no arguments)* | Configuration dialog |

## How it was made

The project was written in Rust with [Claude Code](https://claude.com/claude-code)
acting as the developer, iterating against the real machine: every change was
compiled, launched, and verified with actual screenshots of the running
screensaver and of the configuration dialog before moving on.

- The paper plane is not a downloaded asset: it is modeled **procedurally in
  code** ([src/mesh.rs](src/mesh.rs)) as a classic low-poly dart — two swept
  wings plus a vertical keel, five vertices and three triangles — inspired by
  [this Sketchfab model](https://sketchfab.com/3d-models/paper-plane-low-poly-game-ready-for-free-53c935434bbd4d398bed826b0dd07446).
- The application icon is generated from **the same mesh**: a script projects
  the three triangles with the same flat-shading formula used by the WGSL
  shader, draws them over a dark rounded background, and packs 16/24/32/48/256
  px images into the `.ico` embedded into the binary at build time
  ([build.rs](build.rs)).
- Windowing, the configuration dialog, and monitor/DPI handling use the raw
  Win32 API through the `windows` crate; rendering uses `wgpu`; settings use
  `winreg`. No windowing framework is involved.
- Releases are automated: pushing a version change in `Cargo.toml` to `main`
  makes a GitHub Action build the project, compress the `.scr` at maximum
  ratio, and publish it as release `v{version}`
  ([.github/workflows/release.yml](.github/workflows/release.yml)).

## How it works

**Screensaver protocol.** Windows invokes a `.scr` with standard arguments,
dispatched in [src/main.rs](src/main.rs): `/s` runs full screen, `/c` opens the
configuration dialog, `/p HWND` renders inside the little preview of the
Screen Saver Settings dialog (as a `WS_CHILD` window of the given handle).

**Multi-monitor and HiDPI.** The process declares Per-Monitor DPI Aware V2, so
all coordinates are physical pixels. Monitors are enumerated with
`EnumDisplayMonitors` and each one gets its own borderless topmost window, its
own `wgpu` surface, and an **independent scene** sized to its aspect ratio —
mixed resolutions and DPI scales just work. A single GPU device is shared.
Any key press, mouse click, or mouse movement beyond a few pixels exits.

**Flight model.** Each plane follows an invisible **cubic Bézier curve** whose
control points are sampled inside the camera frustum. The parameter advances
scaled by the inverse of the curve's derivative length, giving near-constant
flight speed. When a curve ends, the next one is chained with **C1
continuity**: it starts at the previous endpoint and its first control point
extends the outgoing tangent, so there are no direction jumps. The plane's
orientation follows the tangent, and it **banks** into turns proportionally to
lateral acceleration (the second derivative projected on the wing axis), with
orientation smoothed by quaternion slerp ([src/scene.rs](src/scene.rs)).

**Rendering.** `wgpu` (Direct3D 12/Vulkan) draws all planes of a monitor in a
single instanced call with 4x MSAA and a depth buffer. The shader applies flat
two-sided lighting — paper looks the same from both sides — plus a subtle
distance fade toward the black background ([src/shader.wgsl](src/shader.wgsl)).

**Configuration.** The dialog is plain Win32 (two trackbars, OK/Cancel, scaled
by DPI). Values are stored in the registry under `HKCU\SOFTWARE\paper_plane`
and clamped on load ([src/config.rs](src/config.rs)). UI text is Spanish when
the Windows display language is Spanish, English otherwise
([src/lang.rs](src/lang.rs)).
