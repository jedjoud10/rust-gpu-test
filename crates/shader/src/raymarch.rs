use shared::*;

use crate::{box_normal, intersection, intersection_faces};

pub struct RaymarchOutput {
    pub position: Vec3,
    pub normal: Vec3,
    pub hit: bool,
    pub last_dist: f32,
    pub iteration_percent: f32,
}



pub fn raymarch(
    ray_start: Vec3,
    ray_dir: Vec3,
    image: &Image!(3D, format=r8ui, sampled=false, depth=false),
) -> RaymarchOutput {
    //let ray_dir = (ray_dir * 200f32).round() / 200f32;
    let inv_dir = ray_dir.recip();
    let mut pos = ray_start + ray_dir * 0.2f32;

    for x in 0..256  {
        let min = pos.floor();
        let max = pos.ceil();

        let int = intersection(pos, inv_dir, min, max);
        let dist = f32::max(int.y, 0.1f32); 
        pos += ray_dir * dist;

        // ((pos + ray_dir * 0.00001).floor()
        if image.read(pos.abs().as_uvec3()) == 1 {


            //let int = intersection(pos, inv_dir, min, max);
            //pos += ray_dir * int.y;

            let normal = Vec3::normalize(pos - min - 0.5f32);
            let mut face = 0u32;
            let shifted = pos;
            intersection_faces(shifted, -inv_dir, min, max, &mut face);
            let normal = box_normal(face, normal);

            let delta = pos - min - 0.5f32;
            let normal = if delta.x.abs() > delta.y.abs() && delta.x.abs() > delta.z.abs() {
                vec3(<f32 as Real>::signum(delta.x), 0.0, 0.0)
            } else if delta.y.abs() > delta.z.abs() {
                vec3(0.0, <f32 as Real>::signum(delta.y), 0.0)
            } else {
                vec3(0.0, 0.0, <f32 as Real>::signum(delta.z))
            };
            
            return RaymarchOutput {
                position: pos,
                normal,
                last_dist: int.y,
                hit: true,
                iteration_percent: x as f32 / 256.0f32
            };
        }
    }

    return RaymarchOutput {
        position: Vec3::ZERO,
        normal: Vec3::ZERO,
        last_dist: 0f32,
        iteration_percent: 0f32,
        hit: false,
    };
}