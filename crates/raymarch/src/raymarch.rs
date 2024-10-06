use shared::*;

use crate::{box_normal, intersection, intersection_faces};

pub struct RaymarchOutput {
    pub position: Vec3,
    pub normal: Vec3,
    pub hit: bool,
    pub last_dist: f32,
    pub iteration_percent: f32,
}

fn indeed(pos: Vec3) -> bool {
    let mut sum = 0f32;

    for i in 1..5 {
        let scale = f32::powf(2f32, i as f32);
        let amplitude = f32::powf(0.5f32, i as f32);
        sum += f32::sin((pos.x + pos.z * 1.2) * 0.40f32 * scale) * amplitude * 6f32;
    }
    
    sum > pos.y
}

pub fn raymarch(
    ray_start: Vec3,
    ray_dir: Vec3,
) -> RaymarchOutput {
    //let ray_dir = (ray_dir * 200f32).round() / 200f32;
    let inv_dir = ray_dir.recip();
    let mut pos = ray_start + ray_dir * 0.2f32;

    for x in 0..256  {
        let min = pos.floor();
        let max = pos.ceil();

        let int = intersection(pos, inv_dir, min, max);

        let dist = f32::max(int.y, 0.001f32); 
        pos += ray_dir * dist;

        if indeed(min) {
            pos -= ray_dir * dist;
            let normal = Vec3::normalize(pos - min - 0.5f32);

            let mut face = 0u32;
            let shifted = pos;
            intersection_faces(shifted, -inv_dir, min, max, &mut face);
            let normal = box_normal(face, normal);
            
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