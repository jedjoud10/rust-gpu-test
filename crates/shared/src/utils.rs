use crate::*;

// gpted from glsl def
#[inline]
pub fn refract(incident: Vec3, normal: Vec3, eta: f32) -> Vec3 {
    let dot_n_i = normal.dot(incident);
    let k = 1.0 - eta * eta * (1.0 - dot_n_i * dot_n_i);

    if k < 0.0 {
        // Total internal reflection: no refraction
        Vec3::ZERO
    } else {
        // Compute the refracted vector
        eta * incident - (eta * dot_n_i + k.sqrt()) * normal
    }
}

// found online here 
// https://math.stackexchange.com/questions/13261/how-to-get-a-reflection-vector
#[inline]
pub fn reflect(ray_dir: Vec3, normal: Vec3) -> Vec3 {
    ray_dir - 2f32 * (ray_dir.dot(normal)) * normal
}