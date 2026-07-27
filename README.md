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
