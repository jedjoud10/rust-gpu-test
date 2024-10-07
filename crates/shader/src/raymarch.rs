use shared::*;

use crate::{box_normal, intersection, intersection_faces};

#[derive(Default)]
pub struct RaymarchOutput {
    pub position: Vec3,
    pub test: Vec3,
    pub local_pos: Vec3,
    pub block_pos: Vec3,
    pub ray_dir: Vec3,
    pub neighbors_bitwise: u32,

    // Couldn't figure out a way to not make this shit. TODO: FIX!!!
    //pub normal: Vec3,

    pub hit: bool,
    pub last_dist: f32,
    pub iteration_percent: f32,
}

pub fn raymarch(
    ray_start: Vec3,
    mut ray_dir: Vec3,
    image: &Image!(3D, format=r8ui, sampled=false, depth=false),
) -> RaymarchOutput {
    let mut inv_dir = ray_dir.recip();
    let mut pos = ray_start + ray_dir * 0.2f32;

    for x in 0..256  {
        let min = pos.floor();
        let max = pos.ceil();
        let int = intersection(pos, inv_dir, min, max);
        let dist = f32::max(int.y, 0.001f32); 
        pos += ray_dir * dist;

        
        if pos.x < -8.0 {
            let reflected = ray_dir - 2f32 * (ray_dir.dot(Vec3::X)) * Vec3::X;
            ray_dir = reflected + (noise::hash33(pos * vec3(42.594, 12.435, 65.945)) - 0.5) * 0.1;
            inv_dir = ray_dir.recip();
        }
        
        if image.read(min.abs().as_uvec3()) == 1 {
            pos -= ray_dir * dist;

            let nx = image.read(min.abs().as_uvec3() + uvec3(1, 0, 0)) as u32;
            let ny = (image.read(min.abs().as_uvec3() + uvec3(0, 1, 0)) as u32) << 1;
            let nz = (image.read(min.abs().as_uvec3() + uvec3(0, 0, 1)) as u32) << 2;
            let nnx = (image.read(min.abs().as_uvec3() - uvec3(1, 0, 0)) as u32) << 3;
            let nny = (image.read(min.abs().as_uvec3() - uvec3(0, 1, 0)) as u32) << 4;
            let nnz = (image.read(min.abs().as_uvec3() - uvec3(0, 0, 1)) as u32) << 5;
            let combined = nx | ny | nz | nnx | nny | nnz;

            return RaymarchOutput {
                local_pos: pos.rem_euclid(Vec3::ONE),
                block_pos: min,
                test: Vec3::ZERO,
                position: pos,
                last_dist: int.y,
                hit: true,
                neighbors_bitwise: combined,
                iteration_percent: x as f32 / 256.0f32,
                ray_dir,
            };
        }
    }

    return RaymarchOutput::default();
}
