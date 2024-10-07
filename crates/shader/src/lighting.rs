use shared::*;
use crate::RaymarchOutput;

pub fn light(input: RaymarchOutput) -> Vec3 {
    //return input.position / 10.0f32;
    //return input.iteration_percent * Vec3::ONE;
    let local = (input.local_pos - input.spherical_normal * 0.01f32).div_euclid(Vec3::ONE / 8.0);
    let sun = vec3(-1.0, -1.0, -1.0).normalize();

    // Calculate simple diffuse color
    let mut diffuse = Vec3::ZERO;
    if input.neighbors_bitwise & 2 == 0 && input.local_pos.y > 0.95 && (input.local_pos.xz() - 0.5).abs().cmplt(Vec2::ONE * 0.95).all() {
        diffuse += vec3(6.0, 43.0, 5.0) / 255.0;
    }
    diffuse += (rng::hash13(input.block_pos) * 0.2 + 0.8) * (rng::hash13(local + input.block_pos) * 0.2 + 0.8) * vec3(45.0, 46.0, 45.0) / 255.0;
    
    // Shade everything and combine em
    let mut color = input.normal.dot(sun).max(0.0) * diffuse * 2.0;
    color += sky(input.position, input.normal) * 0.5 * diffuse;
    color
}

// https://stackoverflow.com/questions/23975555/how-to-calculate-a-ray-plane-intersection
pub fn plane(origin: Vec3, ray: Vec3, normal: Vec3) -> f32 {
    origin.dot(normal) / (normal.dot(ray))
}

pub fn sky(pos: Vec3, dir: Vec3) -> Vec3 {
    let col1 = vec3(155.0, 217.0, 242.0) / 255.0;
    let col2 = vec3(0.0, 39.0, 117.0) / 255.0;
    let mut main = Vec3::lerp(col1, col2, dir.y.max(0.0));
    
    let dist = plane(Vec3::Y * 200.0 - pos, dir, Vec3::Y);
    if dist > 0.0 {
        let pos = pos + dist * dir;
        let val = noise::fbm_simplex_2d(pos.xz().div_euclid(Vec2::ONE * 16.0) * 0.03, 2, 0.5, 1.8).max(0.0);
        main = main.lerp(Vec3::ONE, val);
    }
    
    main
}