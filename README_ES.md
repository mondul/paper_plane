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

> [!NOTE]
> **Advertencia de SmartScreen.** El binario no está firmado digitalmente, así
> que la primera vez que se ejecute Windows puede mostrar el aviso *"Windows
> protegió su PC"*. Haz clic en **Más información** y luego en **Ejecutar de
> todas formas** — solo es necesario hacerlo una vez.
>
> Para evitar sorpresas, se recomienda **ejecutar el `.scr` una vez antes de
> instalarlo** (haz doble clic sobre él; se abrirá el diálogo de
> configuración). Así el aviso de SmartScreen aparece en ese momento y puede
> aceptarse, en lugar de bloquear silenciosamente el protector más tarde,
> cuando Windows intente iniciarlo.

## Argumentos (estándar de protectores de pantalla)

| Argumento | Función |
|-----------|---------|
| `/s`      | Ejecutar a pantalla completa |
| `/c` o `/c:HWND` | Diálogo de configuración |
| `/p HWND` | Vista previa en la miniatura del diálogo de Windows |
| `/w`      | Modo ventana (extra, para depuración; Esc para salir) |
| *(sin argumentos)* | Diálogo de configuración |

## Cómo se hizo

El proyecto fue escrito en Rust con [Claude Code](https://claude.com/claude-code)
como desarrollador, iterando contra la máquina real: cada cambio se compiló,
se ejecutó y se verificó con capturas de pantalla reales del protector en
marcha y del diálogo de configuración antes de continuar.

- El avión de papel no es un recurso descargado: está modelado
  **proceduralmente en código** ([src/mesh.rs](src/mesh.rs)) como el dardo
  low-poly clásico — dos alas en flecha más una quilla vertical, cinco
  vértices y tres triángulos — inspirado en
  [este modelo de Sketchfab](https://sketchfab.com/3d-models/paper-plane-low-poly-game-ready-for-free-53c935434bbd4d398bed826b0dd07446).
- El ícono de la aplicación se genera a partir de **la misma malla**: un
  script proyecta los tres triángulos con la misma fórmula de sombreado plano
  del shader WGSL, los dibuja sobre un fondo oscuro redondeado y empaqueta las
  imágenes de 16/24/32/48/256 px en el `.ico` que se incrusta en el binario al
  compilar ([build.rs](build.rs)).
- Las ventanas, el diálogo de configuración y el manejo de monitores/DPI usan
  la API Win32 directamente a través del crate `windows`; el renderizado usa
  `wgpu`; la configuración usa `winreg`. No hay ningún framework de ventanas.
- Los releases están automatizados: al subir a `main` un cambio de versión en
  `Cargo.toml`, una GitHub Action compila el proyecto, comprime el `.scr` al
  máximo y lo publica como release `v{versión}`
  ([.github/workflows/release.yml](.github/workflows/release.yml)).

## Cómo funciona

**Protocolo de protectores de pantalla.** Windows invoca un `.scr` con
argumentos estándar, que se despachan en [src/main.rs](src/main.rs): `/s`
ejecuta a pantalla completa, `/c` abre el diálogo de configuración y `/p HWND`
renderiza dentro de la miniatura del diálogo de Configuración del protector
(como ventana `WS_CHILD` del handle recibido).

**Multi-monitor y HiDPI.** El proceso se declara Per-Monitor DPI Aware V2, así
que todas las coordenadas son píxeles físicos. Los monitores se enumeran con
`EnumDisplayMonitors` y cada uno recibe su propia ventana sin bordes siempre
visible, su propia superficie `wgpu` y una **escena independiente** ajustada a
su relación de aspecto — resoluciones y escalas de DPI mezcladas funcionan sin
más. Se comparte un único dispositivo de GPU. Cualquier tecla, clic o
movimiento del ratón de más de unos píxeles cierra el protector.

**Modelo de vuelo.** Cada avión sigue una **curva Bézier cúbica** invisible
cuyos puntos de control se muestrean dentro del frustum de la cámara. El
parámetro avanza escalado por el inverso de la longitud de la derivada de la
curva, lo que da una velocidad de vuelo casi constante. Al terminar una curva,
la siguiente se encadena con **continuidad C1**: arranca en el punto final
anterior y su primer punto de control prolonga la tangente de salida, así que
no hay saltos de dirección. La orientación del avión sigue la tangente y se
**inclina en las curvas** (alabeo) en proporción a la aceleración lateral (la
segunda derivada proyectada sobre el eje de las alas), suavizada con slerp de
cuaterniones ([src/scene.rs](src/scene.rs)).

**Renderizado.** `wgpu` (Direct3D 12/Vulkan) dibuja todos los aviones de un
monitor en una sola llamada con instancias, MSAA 4x y búfer de profundidad. El
shader aplica iluminación plana de dos caras — el papel se ve igual por ambos
lados — más un desvanecimiento sutil con la distancia hacia el fondo negro
([src/shader.wgsl](src/shader.wgsl)).

**Configuración.** El diálogo es Win32 puro (dos trackbars, Aceptar/Cancelar,
escalado según DPI). Los valores se guardan en el registro bajo
`HKCU\SOFTWARE\paper_plane` y se acotan al cargar
([src/config.rs](src/config.rs)). Los textos de la interfaz se muestran en
español cuando el idioma de Windows es español y en inglés en cualquier otro
caso ([src/lang.rs](src/lang.rs)).
