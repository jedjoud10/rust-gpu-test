use shared::*;
use crate::{voxel, RaymarchOutput, VoxelType};

//https://github.com/dmnsgn/glsl-tone-map/blob/main/aces.glsl
fn aces(x: Vec3) -> Vec3 {
    const a: f32 = 2.51;
    const b: f32 = 0.03;
    const c: f32 = 2.43;
    const d: f32 = 0.59;
    const e: f32 = 0.14;
    return Vec3::clamp((x * (a * x + b)) / (x * (c * x + d) + e), Vec3::ZERO, Vec3::ONE);
}

pub struct LightingFnParams {
    pub pos: Vec3,
    pub local_pixelated: Vec3,
    pub normal: Vec3,
    pub voxel: VoxelType
}

#[inline]
pub fn light(input: LightingFnParams) -> Vec3 {
    // This should be a parameter but wtv
    let sun = vec3(1.0, 1.0, 1.0).normalize();

    // Rng numbers for each block and pixel within the block
    let block_pos = input.pos.floor();
    

    let block_rng = rng::hash13(block_pos * vec3(15.321, 121.21, 332.5));
    let block_texel_rng = rng::hash13((input.local_pixelated + block_pos * 8.0) * vec3(32.321, 12.321, 53.23));
    
    // Randomize the normal a bit
    let mut normal = input.normal + (block_texel_rng - 0.5) * 0.05;
    normal = normal.normalize();

    /*
    // Calculate simple diffuse color (either green or gray)
    let mut diffuse = if input.neighbors_bitwise & (1 << voxel::neighbor_pos_to_index(ivec3(0, 1, 0))) == 0 && input.local_pixelated.y >= 7.0 {
        vec3(51.0, 89.0, 50.0) / 255.0
    } else {
        vec3(45.0, 46.0, 45.0) / 255.0
    };
    */

    let mut diffuse = input.voxel.diffuse;

    // Vary the colors a bit
    diffuse *= (block_rng * 0.2 + 0.8) * (block_texel_rng * 0.2 + 0.8);

    /*
    let mut ao = 0.0;
    for i in 0..27 {
        if input.neighbors_bitwise & (1 << i) != 0 && i != voxel::neighbor_pos_to_index(IVec3::ZERO) {
            let temp = voxel::neighbor_index_to_pos(i).as_vec3();

            if (temp.normalize().dot(input.normal) > 0.4) {
                ao += temp.normalize().dot(input.spherical_normal).max(0.0);
            }
        }
    }
    */

    //return ao * Vec3::ONE * 0.2;
    
    // Shade everything and combine em
    let mut color = normal.dot(sun).max(0.0) * diffuse * 1.6;
    color += sky(input.pos, normal) * 0.5 * diffuse;
    color
}

// https://stackoverflow.com/questions/23975555/how-to-calculate-a-ray-plane-intersection
#[inline]
pub fn plane(origin: Vec3, ray: Vec3, normal: Vec3) -> f32 {
    origin.dot(normal) / (normal.dot(ray))
}

#[inline]
pub fn sky(ray_start: Vec3, ray_dir: Vec3) -> Vec3 {
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