use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BLACK_BRUSH, EnumDisplayMonitors, GetStockObject, HBRUSH, HDC, HMONITOR,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, GetClientRect, GetCursorPos, IsWindow, MSG, PM_REMOVE, PeekMessageW,
    PostQuitMessage, RegisterClassW, SetCursor, ShowCursor, TranslateMessage, WM_CLOSE,
    WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_MOUSEMOVE,
    WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDOWN, WM_SETCURSOR, WM_SYSKEYDOWN, WM_XBUTTONDOWN,
    WNDCLASSW, WS_CHILD, WS_EX_TOPMOST, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_VISIBLE,
};
use windows::core::w;

use crate::config::Config;
use crate::lang::{texts, to_wide};
use crate::renderer::{Gpu, Renderer, create_instance, create_surface_for_hwnd};
use crate::scene::{InstanceRaw, Scene};

static QUIT: AtomicBool = AtomicBool::new(false);
/// En modo pantalla completa cualquier entrada del usuario cierra el protector.
static INPUT_EXITS: AtomicBool = AtomicBool::new(false);
static LAST_MOUSE: Mutex<Option<(i32, i32)>> = Mutex::new(None);

const MOUSE_THRESHOLD: i32 = 4;

struct WindowCtx {
    hwnd: HWND,
    renderer: Renderer,
    scene: Scene,
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                QUIT.store(true, Ordering::Relaxed);
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_SETCURSOR if INPUT_EXITS.load(Ordering::Relaxed) => {
                SetCursor(None);
                LRESULT(1)
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if INPUT_EXITS.load(Ordering::Relaxed) || wparam.0 as u16 == VK_ESCAPE.0 {
                    QUIT.store(true, Ordering::Relaxed);
                }
                LRESULT(0)
            }
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN | WM_MOUSEWHEEL => {
                if INPUT_EXITS.load(Ordering::Relaxed) {
                    QUIT.store(true, Ordering::Relaxed);
                }
                LRESULT(0)
            }
            WM_MOUSEMOVE if INPUT_EXITS.load(Ordering::Relaxed) => {
                let mut pt = windows::Win32::Foundation::POINT::default();
                if GetCursorPos(&mut pt).is_ok() {
                    let mut last = LAST_MOUSE.lock().unwrap();
                    match *last {
                        Some((x, y)) => {
                            if (pt.x - x).abs() > MOUSE_THRESHOLD
                                || (pt.y - y).abs() > MOUSE_THRESHOLD
                            {
                                QUIT.store(true, Ordering::Relaxed);
                            }
                        }
                        None => *last = Some((pt.x, pt.y)),
                    }
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn register_class() -> windows::core::PCWSTR {
    let class_name = w!("paper_plane_wnd");
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap();
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance.into(),
            hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassW(&wc);
    }
    class_name
}

struct MonitorRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

unsafe extern "system" fn monitor_enum(
    _hmon: HMONITOR,
    _hdc: HDC,
    rect: *mut RECT,
    lparam: LPARAM,
) -> windows::core::BOOL {
    unsafe {
        let list = &mut *(lparam.0 as *mut Vec<MonitorRect>);
        let r = *rect;
        list.push(MonitorRect {
            x: r.left,
            y: r.top,
            w: r.right - r.left,
            h: r.bottom - r.top,
        });
        true.into()
    }
}

fn enumerate_monitors() -> Vec<MonitorRect> {
    let mut monitors: Vec<MonitorRect> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum),
            LPARAM(&mut monitors as *mut _ as isize),
        );
    }
    if monitors.is_empty() {
        monitors.push(MonitorRect {
            x: 0,
            y: 0,
            w: 1280,
            h: 720,
        });
    }
    monitors
}

fn client_size(hwnd: HWND) -> (u32, u32) {
    let mut rect = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut rect);
    }
    (
        (rect.right - rect.left).max(1) as u32,
        (rect.bottom - rect.top).max(1) as u32,
    )
}

/// Pantalla completa en todos los monitores (argumento /s).
pub fn run_fullscreen() {
    INPUT_EXITS.store(true, Ordering::Relaxed);
    let class = register_class();
    let hinstance = unsafe { GetModuleHandleW(None).unwrap() };

    let mut hwnds = Vec::new();
    for m in enumerate_monitors() {
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOPMOST,
                class,
                w!("Paper Plane"),
                WS_POPUP | WS_VISIBLE,
                m.x,
                m.y,
                m.w,
                m.h,
                None,
                None,
                Some(hinstance.into()),
                None,
            )
        };
        if let Ok(hwnd) = hwnd {
            hwnds.push(hwnd);
        }
    }
    unsafe {
        let _ = ShowCursor(false);
    }
    run_loop(hwnds, None);
}

/// Vista previa dentro de la miniatura del diálogo de Windows (argumento /p).
pub fn run_preview(parent: HWND) {
    if unsafe { !IsWindow(Some(parent)).as_bool() } {
        return;
    }
    let class = register_class();
    let hinstance = unsafe { GetModuleHandleW(None).unwrap() };
    let (w, h) = client_size(parent);
    let hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            class,
            w!("Paper Plane Preview"),
            WS_CHILD | WS_VISIBLE,
            0,
            0,
            w as i32,
            h as i32,
            Some(parent),
            None,
            Some(hinstance.into()),
            None,
        )
    };
    if let Ok(hwnd) = hwnd {
        run_loop(vec![hwnd], Some(parent));
    }
}

/// Modo ventana para depuración (argumento /w). Esc para salir.
pub fn run_windowed() {
    let class = register_class();
    let hinstance = unsafe { GetModuleHandleW(None).unwrap() };
    let title_w = to_wide(texts().windowed_title);
    let hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            class,
            windows::core::PCWSTR(title_w.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1280,
            800,
            None,
            None,
            Some(hinstance.into()),
            None,
        )
    };
    if let Ok(hwnd) = hwnd {
        run_loop(vec![hwnd], None);
    }
}

fn run_loop(hwnds: Vec<HWND>, preview_parent: Option<HWND>) {
    if hwnds.is_empty() {
        return;
    }
    let cfg = Config::load();
    let speed = cfg.speed_units();

    let instance = create_instance();
    let mut surfaces = Vec::new();
    for &hwnd in &hwnds {
        match create_surface_for_hwnd(&instance, hwnd) {
            Ok(s) => surfaces.push(s),
            Err(_) => return,
        }
    }
    let gpu = Gpu::new(&instance, &surfaces[0]);

    let mut windows_ctx: Vec<WindowCtx> = Vec::new();
    for (i, (hwnd, surface)) in hwnds.into_iter().zip(surfaces).enumerate() {
        let (w, h) = client_size(hwnd);
        let renderer = Renderer::new(&gpu, surface, w, h, cfg.plane_count);
        let seed = 0x9E3779B97F4A7C15u64.wrapping_mul(i as u64 + 1)
            ^ std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42);
        let scene = Scene::new(renderer.aspect(), cfg.plane_count, seed);
        windows_ctx.push(WindowCtx {
            hwnd,
            renderer,
            scene,
        });
    }

    let mut last_frame = Instant::now();
    let mut instances: Vec<InstanceRaw> = Vec::new();
    let mut msg = MSG::default();

    loop {
        unsafe {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    QUIT.store(true, Ordering::Relaxed);
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        if QUIT.load(Ordering::Relaxed) {
            break;
        }
        if let Some(parent) = preview_parent {
            if unsafe { !IsWindow(Some(parent)).as_bool() } {
                break;
            }
        }

        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(0.1);
        last_frame = now;

        for ctx in &mut windows_ctx {
            // Reaccionar a cambios de tamaño (relevante en modo ventana).
            let (w, h) = client_size(ctx.hwnd);
            if (w, h) != ctx.renderer.size() {
                ctx.renderer.resize(&gpu, w, h);
                ctx.scene.set_aspect(ctx.renderer.aspect());
            }
            ctx.scene.update(dt, speed);
            ctx.scene.instances(&mut instances);
            ctx.renderer.render(&gpu, &instances);
        }
    }

    for ctx in &windows_ctx {
        unsafe {
            if IsWindow(Some(ctx.hwnd)).as_bool() {
                let _ = DestroyWindow(ctx.hwnd);
            }
        }
    }
}
