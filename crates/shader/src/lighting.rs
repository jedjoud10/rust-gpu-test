use shared::*;
use crate::RaymarchOutput;

//https://github.com/dmnsgn/glsl-tone-map/blob/main/aces.glsl
fn aces(x: Vec3) -> Vec3 {
    const a: f32 = 2.51;
    const b: f32 = 0.03;
    const c: f32 = 2.43;
    const d: f32 = 0.59;
    const e: f32 = 0.14;
    return Vec3::clamp((x * (a * x + b)) / (x * (c * x + d) + e), Vec3::ZERO, Vec3::ONE);
}

pub fn light(input: RaymarchOutput) -> Vec3 {
    //return input.normal;
    //return input.position / 10.0f32;
    //return input.iteration_percent * Vec3::ONE;

    // This should be a parameter but wtv
    let sun = vec3(1.0, 1.0, 1.0).normalize();

    // Rng numbers for each block and pixel within the block
    let block_rng = rng::hash13(input.block_pos * vec3(15.321, 121.21, 332.5));
    let block_texel_rng = rng::hash13((input.local_pixelated + input.block_pos * 8.0) * vec3(32.321, 12.321, 53.23));
    
    // Randomize the normal a bit
    let mut normal = input.normal + (block_texel_rng - 0.5) * 0.1;
    normal = normal.normalize();

    // Calculate simple diffuse color (either green or gray)
    let mut diffuse = if input.neighbors_bitwise & 2 == 0 && input.local_pixelated.y >= 7.0 {
        vec3(51.0, 89.0, 50.0) / 255.0
    } else {
        vec3(45.0, 46.0, 45.0) / 255.0
    };

    // Vary the colors a bit
    diffuse *= (block_rng * 0.2 + 0.8) * (block_texel_rng * 0.2 + 0.8);
    
    // Shade everything and combine em
    let mut color = normal.dot(sun).max(0.0) * diffuse * 2.2;
    color += sky(RaymarchOutput {
        ray_dir: normal,
        ray_start: input.position,
        ..Default::default()
    }) * 0.5 * diffuse;
    color
}

// https://stackoverflow.com/questions/23975555/how-to-calculate-a-ray-plane-intersection
pub fn plane(origin: Vec3, ray: Vec3, normal: Vec3) -> f32 {
    origin.dot(normal) / (normal.dot(ray))
}

pub fn sky(input: RaymarchOutput) -> Vec3 {
    let pos = input.ray_start;
    let dir = input.ray_dir;
    
    let col1 = vec3(155.0, 217.0, 242.0) / 255.0;
    let col2 = vec3(0.0, 39.0, 117.0) / 255.0;
    let mut main = Vec3::lerp(col1, col2, dir.y.max(0.0));
    
    let dist = plane(Vec3::Y * 200.0 - pos, dir, Vec3::Y);
    if dist > 0.0 {
        let pos = pos + dist * dir;
        //let val = noise::fbm_simplex_2d(pos.xz().div_euclid(Vec2::ONE * 16.0) * 0.03, 2, 0.5, 1.8).max(0.0) * 0.6;
        //main = main.lerp(Vec3::ONE, val);
    }
    
    main.clamp(Vec3::ZERO, Vec3::ONE)
}