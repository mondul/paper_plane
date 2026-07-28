use windows::Win32::Globalization::GetUserDefaultUILanguage;

/// Textos de la interfaz. Se muestran en español si el idioma de la interfaz
/// de Windows es español; en inglés en cualquier otro caso.
pub struct Texts {
    pub config_title: &'static str,
    pub plane_count: &'static str,
    pub speed: &'static str,
    pub ok: &'static str,
    pub cancel: &'static str,
    pub windowed_title: &'static str,
}

const ES: Texts = Texts {
    config_title: "Aviones de papel — Configuración",
    plane_count: "Cantidad de aviones:",
    speed: "Velocidad:",
    ok: "Aceptar",
    cancel: "Cancelar",
    windowed_title: "Aviones de papel (modo ventana)",
};

const EN: Texts = Texts {
    config_title: "Paper Planes — Settings",
    plane_count: "Number of planes:",
    speed: "Speed:",
    ok: "OK",
    cancel: "Cancel",
    windowed_title: "Paper Planes (windowed mode)",
};

const LANG_SPANISH: u16 = 0x0A;

pub fn texts() -> &'static Texts {
    let langid = unsafe { GetUserDefaultUILanguage() };
    if langid & 0x3FF == LANG_SPANISH {
        &ES
    } else {
        &EN
    }
}

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
