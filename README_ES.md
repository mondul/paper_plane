# paper_plane

*Read in English: [README.md](README.md)*

Protector de pantalla para Windows (.scr) escrito en Rust: aviones de papel
low-poly que vuelan siguiendo curvas Bézier aleatorias sobre un fondo negro.

- Soporte multi-monitor (una escena independiente por monitor).
- Compatible con HiDPI (Per-Monitor DPI Aware V2, renderizado a resolución física).
- Renderizado con wgpu (Direct3D 12 / Vulkan) con MSAA 4x.
- Diálogo de configuración nativo: cantidad de aviones (1–50) y velocidad (1–10).
- La configuración se guarda en `HKCU\SOFTWARE\paper_plane`
  (valores `PlaneCount` y `Speed`, DWORD).

## Compilación

```
cargo build --release
copy target\release\paper_plane.exe paper_plane.scr
```

## Instalación

1. Clic derecho sobre `paper_plane.scr` → **Instalar**, o
2. Copiar `paper_plane.scr` a `C:\Windows\System32` (o cualquier carpeta) y
   seleccionarlo en Configuración → Personalización → Pantalla de bloqueo →
   Protector de pantalla.

## Argumentos (estándar de protectores de pantalla)

| Argumento | Función |
|-----------|---------|
| `/s`      | Ejecutar a pantalla completa |
| `/c` o `/c:HWND` | Diálogo de configuración |
| `/p HWND` | Vista previa en la miniatura del diálogo de Windows |
| `/w`      | Modo ventana (extra, para depuración; Esc para salir) |
| *(sin argumentos)* | Diálogo de configuración |
