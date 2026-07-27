use std::cell::RefCell;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, DEFAULT_PITCH,
    DeleteObject, FF_DONTCARE, FW_NORMAL, OUT_DEFAULT_PRECIS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{
    ICC_BAR_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx, TBM_SETPOS, TBM_SETRANGE,
};
use windows::Win32::UI::HiDpi::{AdjustWindowRectExForDpi, GetDpiForSystem};
use windows::Win32::UI::WindowsAndMessaging::{
    BS_DEFPUSHBUTTON, BS_PUSHBUTTON, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, GetMessageW, GetSystemMetrics, HMENU, IsDialogMessageW, MSG,
    PostQuitMessage, RegisterClassW, SM_CXSCREEN, SM_CYSCREEN, SendMessageW, SetWindowTextW,
    TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_DESTROY, WM_HSCROLL,
    WM_SETFONT, WNDCLASSW, WS_CAPTION, WS_CHILD, WS_EX_DLGMODALFRAME, WS_SYSMENU, WS_TABSTOP,
    WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

use crate::config::{Config, MAX_PLANES, MAX_SPEED, MIN_PLANES, MIN_SPEED};

const ID_OK: usize = 1; // IDOK
const ID_CANCEL: usize = 2; // IDCANCEL

// Estilos y mensajes de controles que el crate `windows` no expone.
const TBS_HORZ: u32 = 0x0000;
const TBS_AUTOTICKS: u32 = 0x0001;
const TBM_GETPOS: u32 = 0x0400; // WM_USER
const SS_RIGHT: u32 = 0x0002;

struct Ui {
    slider_count: HWND,
    slider_speed: HWND,
    value_count: HWND,
    value_speed: HWND,
}

thread_local! {
    static UI: RefCell<Option<Ui>> = const { RefCell::new(None) };
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn slider_pos(slider: HWND) -> u32 {
    unsafe { SendMessageW(slider, TBM_GETPOS, None, None).0 as u32 }
}

fn update_value_labels() {
    UI.with_borrow(|ui| {
        if let Some(ui) = ui {
            let count = slider_pos(ui.slider_count);
            let speed = slider_pos(ui.slider_speed);
            unsafe {
                let _ = SetWindowTextW(
                    ui.value_count,
                    PCWSTR(to_wide(&count.to_string()).as_ptr()),
                );
                let _ = SetWindowTextW(
                    ui.value_speed,
                    PCWSTR(to_wide(&speed.to_string()).as_ptr()),
                );
            }
        }
    });
}

unsafe extern "system" fn dlgproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_HSCROLL => {
                update_value_labels();
                LRESULT(0)
            }
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as usize;
                match id {
                    ID_OK => {
                        UI.with_borrow(|ui| {
                            if let Some(ui) = ui {
                                let cfg = Config {
                                    plane_count: slider_pos(ui.slider_count),
                                    speed: slider_pos(ui.slider_speed),
                                };
                                cfg.save();
                            }
                        });
                        let _ = DestroyWindow(hwnd);
                    }
                    ID_CANCEL => {
                        let _ = DestroyWindow(hwnd);
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Diálogo de configuración (argumento /c). Guarda en HKCU\SOFTWARE\paper_plane.
pub fn run_config(parent: Option<HWND>) {
    unsafe {
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_BAR_CLASSES,
        };
        let _ = InitCommonControlsEx(&icc);

        let hinstance = GetModuleHandleW(None).unwrap();
        let class_name = w!("paper_plane_cfg");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(dlgproc),
            hInstance: hinstance.into(),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(
                (windows::Win32::Graphics::Gdi::COLOR_BTNFACE.0 as usize + 1) as *mut _,
            ),
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassW(&wc);

        let cfg = Config::load();

        // Layout en píxeles lógicos (96 dpi), escalado según el DPI del sistema.
        let dpi = GetDpiForSystem();
        let s = |v: i32| -> i32 { v * dpi as i32 / 96 };

        let client_w = s(340);
        let client_h = s(202);

        let style = WS_CAPTION | WS_SYSMENU;
        let ex_style = WS_EX_DLGMODALFRAME;
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: client_w,
            bottom: client_h,
        };
        let _ = AdjustWindowRectExForDpi(&mut rect, style, false, ex_style, dpi);
        let win_w = rect.right - rect.left;
        let win_h = rect.bottom - rect.top;
        let x = (GetSystemMetrics(SM_CXSCREEN) - win_w) / 2;
        let y = (GetSystemMetrics(SM_CYSCREEN) - win_h) / 2;

        let hwnd = CreateWindowExW(
            ex_style,
            class_name,
            w!("Aviones de papel — Configuración"),
            style | WS_VISIBLE,
            x,
            y,
            win_w,
            win_h,
            parent,
            None,
            Some(hinstance.into()),
            None,
        );
        let Ok(hwnd) = hwnd else { return };

        let font = CreateFontW(
            -(9 * dpi as i32) / 72,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32 | (FF_DONTCARE.0 as u32) << 4,
            w!("Segoe UI"),
        );

        let make = |class: PCWSTR,
                    text: PCWSTR,
                    style: WINDOW_STYLE,
                    x: i32,
                    y: i32,
                    w: i32,
                    h: i32,
                    id: usize|
         -> HWND {
            let child = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class,
                text,
                WS_CHILD | WS_VISIBLE | style,
                s(x),
                s(y),
                s(w),
                s(h),
                Some(hwnd),
                if id != 0 { Some(HMENU(id as *mut _)) } else { None },
                Some(hinstance.into()),
                None,
            )
            .unwrap();
            if !font.is_invalid() {
                SendMessageW(
                    child,
                    WM_SETFONT,
                    Some(WPARAM(font.0 as usize)),
                    Some(LPARAM(1)),
                );
            }
            child
        };

        let static_class = w!("STATIC");
        let button_class = w!("BUTTON");
        let trackbar_class = w!("msctls_trackbar32");

        make(
            static_class,
            w!("Cantidad de aviones:"),
            WINDOW_STYLE(0),
            16,
            14,
            200,
            20,
            0,
        );
        let value_count = make(
            static_class,
            w!(""),
            WINDOW_STYLE(SS_RIGHT),
            216,
            14,
            92,
            20,
            0,
        );
        let slider_count = make(
            trackbar_class,
            w!(""),
            WINDOW_STYLE(TBS_HORZ | TBS_AUTOTICKS) | WS_TABSTOP,
            12,
            38,
            316,
            32,
            0,
        );

        make(
            static_class,
            w!("Velocidad:"),
            WINDOW_STYLE(0),
            16,
            86,
            200,
            20,
            0,
        );
        let value_speed = make(
            static_class,
            w!(""),
            WINDOW_STYLE(SS_RIGHT),
            216,
            86,
            92,
            20,
            0,
        );
        let slider_speed = make(
            trackbar_class,
            w!(""),
            WINDOW_STYLE(TBS_HORZ | TBS_AUTOTICKS) | WS_TABSTOP,
            12,
            110,
            316,
            32,
            0,
        );

        make(
            button_class,
            w!("Aceptar"),
            WINDOW_STYLE(BS_DEFPUSHBUTTON as u32) | WS_TABSTOP,
            148,
            160,
            84,
            28,
            ID_OK,
        );
        make(
            button_class,
            w!("Cancelar"),
            WINDOW_STYLE(BS_PUSHBUTTON as u32) | WS_TABSTOP,
            240,
            160,
            84,
            28,
            ID_CANCEL,
        );

        // Rangos y posiciones iniciales de los sliders.
        let make_lparam = |lo: u32, hi: u32| LPARAM(((hi << 16) | (lo & 0xFFFF)) as isize);
        SendMessageW(
            slider_count,
            TBM_SETRANGE,
            Some(WPARAM(1)),
            Some(make_lparam(MIN_PLANES, MAX_PLANES)),
        );
        SendMessageW(
            slider_count,
            TBM_SETPOS,
            Some(WPARAM(1)),
            Some(LPARAM(cfg.plane_count as isize)),
        );
        SendMessageW(
            slider_speed,
            TBM_SETRANGE,
            Some(WPARAM(1)),
            Some(make_lparam(MIN_SPEED, MAX_SPEED)),
        );
        SendMessageW(
            slider_speed,
            TBM_SETPOS,
            Some(WPARAM(1)),
            Some(LPARAM(cfg.speed as isize)),
        );

        UI.set(Some(Ui {
            slider_count,
            slider_speed,
            value_count,
            value_speed,
        }));
        update_value_labels();

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        if !font.is_invalid() {
            let _ = DeleteObject(font.into());
        }
    }
}
