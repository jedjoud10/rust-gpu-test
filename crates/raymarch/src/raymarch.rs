use shared::*;

use crate::{box_normal, intersection, intersection_faces};

pub struct RaymarchOutput {
    pub position: Vec3,
    pub normal: Vec3,
    pub hit: bool,
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
    let inv_dir = ray_dir.recip();
    let mut pos = ray_start + ray_dir * 0.2f32;

    for x in 0..64 {
        let min = (pos * 1f32).floor();
        let max = (pos * 1f32).ceil();

        let int = intersection(pos, inv_dir, min, max);
        let dist = f32::max(int.y, 0.001f32);
        pos += ray_dir * dist;

        let mut face = 0u32;
        intersection_faces(pos + ray_dir * 0.01f32, -inv_dir, min, max, &mut face);

        if indeed(min) {
            let normal = box_normal(face, -ray_dir);
            return RaymarchOutput {
                position: pos,
                normal,
                hit: true,
            };
        }

        /*
        let dis = dist(pos);
        pos += ray_dir * dis;

        if dis < 0.001 {
            //color = pos;
            break;
        }
        */

    }

    return RaymarchOutput {
        position: Vec3::ZERO,
        normal: Vec3::ZERO,
        hit: false,
    };
}