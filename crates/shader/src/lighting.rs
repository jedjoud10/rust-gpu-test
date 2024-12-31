use shared::*;
use crate::{voxel::{self}, RaymarchOutput, VoxelLightingData, VoxelType};

//https://github.com/dmnsgn/glsl-tone-map/blob/main/aces.glsl
fn aces(x: Vec3) -> Vec3 {
    const a: f32 = 2.51;
    const b: f32 = 0.03;
    const c: f32 = 2.43;
    const d: f32 = 0.59;
    const e: f32 = 0.14;
    return Vec3::clamp((x * (a * x + b)) / (x * (c * x + d) + e), Vec3::ZERO, Vec3::ONE);
}

#[inline]
pub fn light(pos: Vec3, local: Vec3, mut normal: Vec3) -> Vec3 {
    // This should be a parameter but wtv
    let sun = vec3(1.0, 1.0, 1.0).normalize();

    // Rng numbers for each block and pixel within the block
    let block_pos = pos.floor();
    let local_pixelated = local.div_euclid(Vec3::ONE / 8.0);
    let block_rng = rng::hash13(block_pos * vec3(15.321, 121.21, 332.5));
    let block_texel_rng = rng::hash13((local_pixelated + block_pos * 8.0) * vec3(32.321, 12.321, 53.23));
    
    let mut data = VoxelLightingData {
        pos: &pos,
        local_pixelated: &local_pixelated,
        normal: &mut normal,
        block_rng: &block_rng,
        block_texel_rng: &block_texel_rng,
    };

    *data.normal = (*data.normal + (block_texel_rng - 0.5) * 0.05).normalize();

    let diffuse = (data.block_rng * 0.2 + 0.8) * (data.block_texel_rng * 0.2 + 0.8) * (if data.normal.y > 0.8 {
        vec3(51.0, 89.0, 50.0) / 255.0
    } else {
        vec3(45.0, 46.0, 45.0) / 255.0
    });

    let mut color = data.normal.dot(sun).max(0.0) * diffuse * 1.6;
    color += skybox(*data.pos, normal) * 0.5 * diffuse;
    color
}

// https://stackoverflow.com/questions/23975555/how-to-calculate-a-ray-plane-intersection
#[inline]
pub fn plane(origin: Vec3, ray: Vec3, normal: Vec3) -> f32 {
    origin.dot(normal) / (normal.dot(ray))
}

#[inline]
pub fn skybox(ray_start: Vec3, ray_dir: Vec3) -> Vec3 {
    return Vec3::ZERO;

    let pos = ray_start;
    let dir = ray_dir;
    
    let col1 = vec3(155.0, 217.0, 242.0) / 255.0;
    let col2 = vec3(0.0, 39.0, 117.0) / 255.0;
    let mut main = Vec3::lerp(col1, col2, dir.y.max(0.0));
    
    let dist = plane(Vec3::Y * 200.0 - pos, dir, Vec3::Y);
    if dist > 0.0 {
        let pos = pos + dist * dir;
        let val = noise::fbm_simplex_2d(pos.xz().div_euclid(Vec2::ONE * 16.0) * 0.03, 2, 0.5, 1.8).max(0.0) * 0.6 * (1.0 - (pos.xz().distance_squared(ray_start.xz()) - 30000000.0).clamp(0.0, 1.0));
        main = main.lerp(Vec3::ONE, val);
    }
    
    main.clamp(Vec3::ZERO, Vec3::ONE)
}