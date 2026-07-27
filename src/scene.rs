use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Mat4, Quat, Vec3};

const FOV_Y: f32 = std::f32::consts::FRAC_PI_3; // 60°
const Z_NEAR_BAND: f32 = -9.0;
const Z_FAR_BAND: f32 = -26.0;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
}

struct Plane {
    ctrl: [Vec3; 4],
    t: f32,
    orientation: Quat,
    scale: f32,
}

pub struct Scene {
    planes: Vec<Plane>,
    aspect: f32,
    rng: fastrand::Rng,
}

fn bezier(c: &[Vec3; 4], t: f32) -> Vec3 {
    let u = 1.0 - t;
    c[0] * (u * u * u)
        + c[1] * (3.0 * u * u * t)
        + c[2] * (3.0 * u * t * t)
        + c[3] * (t * t * t)
}

fn bezier_d1(c: &[Vec3; 4], t: f32) -> Vec3 {
    let u = 1.0 - t;
    (c[1] - c[0]) * (3.0 * u * u)
        + (c[2] - c[1]) * (6.0 * u * t)
        + (c[3] - c[2]) * (3.0 * t * t)
}

fn bezier_d2(c: &[Vec3; 4], t: f32) -> Vec3 {
    let u = 1.0 - t;
    (c[2] - c[1] * 2.0 + c[0]) * (6.0 * u) + (c[3] - c[2] * 2.0 + c[1]) * (6.0 * t)
}

impl Scene {
    pub fn new(aspect: f32, plane_count: u32, seed: u64) -> Self {
        let mut scene = Self {
            planes: Vec::new(),
            aspect,
            rng: fastrand::Rng::with_seed(seed),
        };
        for _ in 0..plane_count {
            let p0 = scene.random_point();
            let p1 = scene.random_point();
            let p2 = scene.random_point();
            let p3 = scene.random_point();
            let ctrl = [p0, p1, p2, p3];
            let t = scene.rng.f32();
            let dir = bezier_d1(&ctrl, t).normalize_or(Vec3::NEG_Z);
            let scale = 0.9 + scene.rng.f32() * 0.4;
            scene.planes.push(Plane {
                ctrl,
                t,
                orientation: orientation_for(dir, 0.0),
                scale,
            });
        }
        scene
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    /// Punto aleatorio dentro del frustum de la cámara (cámara en el origen
    /// mirando hacia -Z), a una profundidad entre las dos bandas.
    fn random_point(&mut self) -> Vec3 {
        let z = Z_NEAR_BAND + (Z_FAR_BAND - Z_NEAR_BAND) * self.rng.f32();
        let half_h = (FOV_Y * 0.5).tan() * (-z) * 0.85;
        let half_w = half_h * self.aspect;
        let x = (self.rng.f32() * 2.0 - 1.0) * half_w;
        let y = (self.rng.f32() * 2.0 - 1.0) * half_h * 0.9;
        Vec3::new(x, y, z)
    }

    pub fn update(&mut self, dt: f32, speed: f32) {
        for i in 0..self.planes.len() {
            // Avance con velocidad aproximadamente constante en arco.
            let (t, ctrl) = {
                let p = &self.planes[i];
                (p.t, p.ctrl)
            };
            let vel = bezier_d1(&ctrl, t).length().max(0.05);
            let mut new_t = t + speed * dt / vel;

            let mut new_ctrl = ctrl;
            if new_t >= 1.0 {
                // Nueva curva encadenada con continuidad C1: arranca donde
                // terminó la anterior y conserva la dirección de la tangente.
                let p0 = ctrl[3];
                let out_dir = (ctrl[3] - ctrl[2]).normalize_or(Vec3::NEG_Z);
                let p1 = p0 + out_dir * (2.0 + self.rng.f32() * 4.0);
                let p2 = self.random_point();
                let p3 = self.random_point();
                new_ctrl = [p0, p1, p2, p3];
                new_t = 0.0;
            }

            let pos_dir = bezier_d1(&new_ctrl, new_t);
            let dir = pos_dir.normalize_or(Vec3::NEG_Z);

            // Alabeo (banking): inclinarse hacia el interior de la curva según
            // la aceleración lateral.
            let accel = bezier_d2(&new_ctrl, new_t);
            let right = dir.cross(Vec3::Y).normalize_or(Vec3::X);
            let lateral = accel.dot(right);
            let bank = (-lateral * 0.05).clamp(-1.1, 1.1);

            let target = orientation_for(dir, bank);
            let p = &mut self.planes[i];
            p.ctrl = new_ctrl;
            p.t = new_t;
            let blend = 1.0 - (-6.0 * dt).exp();
            p.orientation = p.orientation.slerp(target, blend);
        }
    }

    pub fn instances(&self, out: &mut Vec<InstanceRaw>) {
        out.clear();
        for p in &self.planes {
            let pos = bezier(&p.ctrl, p.t);
            let model =
                Mat4::from_scale_rotation_translation(Vec3::splat(p.scale), p.orientation, pos);
            out.push(InstanceRaw {
                model: model.to_cols_array_2d(),
            });
        }
    }
}

/// Orientación cuya dirección de vuelo (morro, -Z del modelo) apunta hacia
/// `dir`, con un ángulo de alabeo `bank` alrededor del eje de vuelo.
fn orientation_for(dir: Vec3, bank: f32) -> Quat {
    let right = dir.cross(Vec3::Y).normalize_or(Vec3::X);
    let up = right.cross(dir).normalize_or(Vec3::Y);
    // Base ortonormal derecha: (right, up, -forward).
    let rot = Quat::from_mat3(&Mat3::from_cols(right, up, -dir));
    Quat::from_axis_angle(dir, bank) * rot
}
