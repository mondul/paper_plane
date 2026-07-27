struct Globals {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) m0: vec4<f32>,
    @location(3) m1: vec4<f32>,
    @location(4) m2: vec4<f32>,
    @location(5) m3: vec4<f32>,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) world: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    let model = mat4x4<f32>(in.m0, in.m1, in.m2, in.m3);
    let world = model * vec4<f32>(in.position, 1.0);
    var out: VsOut;
    out.clip = globals.view_proj * world;
    out.normal = (model * vec4<f32>(in.normal, 0.0)).xyz;
    out.world = world.xyz;
    return out;
}

@fragment
fn fs_main(in: VsOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    var n = normalize(in.normal);
    if (!front) {
        n = -n;
    }
    let light = normalize(vec3<f32>(0.35, 0.8, 0.45));
    let base = vec3<f32>(0.93, 0.93, 0.97);
    let diffuse = max(dot(n, light), 0.0);
    // Atenuación con la distancia para dar sensación de profundidad
    // sobre el fondo negro (la cámara está en el origen).
    let dist = length(in.world);
    let fade = clamp(1.6 - dist / 22.0, 0.25, 1.0);
    let color = base * (0.18 + 0.82 * diffuse) * fade;
    return vec4<f32>(color, 1.0);
}
