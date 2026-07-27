use bytemuck::{Pod, Zeroable};
use glam::Vec3;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

/// Avión de papel low-poly estilo "dardo" clásico, modelado por código.
/// El morro apunta hacia -Z (convención de "forward" usada por la escena),
/// +Y es arriba. Longitud ~1.1 unidades.
pub fn paper_plane() -> Vec<Vertex> {
    let nose = Vec3::new(0.0, 0.0, -0.55);
    let tail_top = Vec3::new(0.0, 0.0, 0.55); // final del lomo (pliegue central)
    let tip_l = Vec3::new(-0.50, 0.12, 0.55); // punta del ala izquierda
    let tip_r = Vec3::new(0.50, 0.12, 0.55); // punta del ala derecha
    let keel = Vec3::new(0.0, -0.22, 0.55); // quilla inferior (cuerpo plegado)

    // Cada cara es un triángulo con normal plana (flat shading). El pipeline
    // dibuja sin culling y el shader ilumina ambos lados, como papel real.
    let faces: [[Vec3; 3]; 3] = [
        [nose, tip_l, tail_top],  // ala izquierda
        [nose, tail_top, tip_r],  // ala derecha
        [nose, keel, tail_top],   // quilla vertical
    ];

    let mut vertices = Vec::with_capacity(faces.len() * 3);
    for face in faces {
        let normal = (face[1] - face[0]).cross(face[2] - face[0]).normalize();
        for p in face {
            vertices.push(Vertex {
                position: p.to_array(),
                normal: normal.to_array(),
            });
        }
    }
    vertices
}
