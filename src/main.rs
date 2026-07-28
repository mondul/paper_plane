#![windows_subsystem = "windows"]

mod config;
mod dialog;
mod lang;
mod mesh;
mod renderer;
mod scene;
mod screensaver;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};

enum Mode {
    /// Diálogo de configuración (opcionalmente con ventana padre).
    Config(Option<HWND>),
    /// Protector a pantalla completa en todos los monitores.
    Run,
    /// Vista previa dentro de la miniatura del diálogo de Windows.
    Preview(HWND),
    /// Modo ventana para depuración (no es parte del estándar .scr).
    Windowed,
}

fn parse_args(args: &[String]) -> Mode {
    let Some(first) = args.first() else {
        return Mode::Config(None);
    };
    let arg = first.to_ascii_lowercase();
    let arg = arg.trim_start_matches(['/', '-']);

    let parse_hwnd = |s: &str| -> Option<HWND> { s.parse::<isize>().ok().map(|v| HWND(v as *mut _)) };

    match arg.chars().next() {
        Some('s') => Mode::Run,
        Some('w') => Mode::Windowed,
        Some('p') => {
            // "/p HWND" o "/p:HWND"
            let hwnd = arg
                .split(':')
                .nth(1)
                .and_then(parse_hwnd)
                .or_else(|| args.get(1).and_then(|s| parse_hwnd(s)));
            match hwnd {
                Some(h) => Mode::Preview(h),
                None => Mode::Config(None),
            }
        }
        Some('c') => {
            // "/c:HWND" o "/c HWND" (el padre es opcional)
            let hwnd = arg
                .split(':')
                .nth(1)
                .and_then(parse_hwnd)
                .or_else(|| args.get(1).and_then(|s| parse_hwnd(s)));
            Mode::Config(hwnd)
        }
        // "/a HWND" (cambio de contraseña, obsoleto): no hacemos nada.
        Some('a') => std::process::exit(0),
        _ => Mode::Config(None),
    }
}

fn main() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_args(&args) {
        Mode::Config(parent) => dialog::run_config(parent),
        Mode::Run => screensaver::run_fullscreen(),
        Mode::Preview(parent) => screensaver::run_preview(parent),
        Mode::Windowed => screensaver::run_windowed(),
    }
}
