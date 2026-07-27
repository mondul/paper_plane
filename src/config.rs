use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

const KEY_PATH: &str = "SOFTWARE\\paper_plane";

pub const MIN_PLANES: u32 = 1;
pub const MAX_PLANES: u32 = 50;
pub const MIN_SPEED: u32 = 1;
pub const MAX_SPEED: u32 = 10;

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub plane_count: u32,
    pub speed: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            plane_count: 12,
            speed: 5,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut cfg = Self::default();
        if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(KEY_PATH) {
            if let Ok(v) = key.get_value::<u32, _>("PlaneCount") {
                cfg.plane_count = v;
            }
            if let Ok(v) = key.get_value::<u32, _>("Speed") {
                cfg.speed = v;
            }
        }
        cfg.clamp();
        cfg
    }

    pub fn save(&self) {
        if let Ok((key, _)) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(KEY_PATH) {
            let _ = key.set_value("PlaneCount", &self.plane_count);
            let _ = key.set_value("Speed", &self.speed);
        }
    }

    fn clamp(&mut self) {
        self.plane_count = self.plane_count.clamp(MIN_PLANES, MAX_PLANES);
        self.speed = self.speed.clamp(MIN_SPEED, MAX_SPEED);
    }

    /// Velocidad de vuelo en unidades de mundo por segundo.
    pub fn speed_units(&self) -> f32 {
        1.5 + self.speed as f32 * 1.1
    }
}
