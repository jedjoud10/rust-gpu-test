#[allow(unused_imports)]
#[cfg_attr(target_arch = "spirv", no_std)]

use shared::*;


pub fn intersection_faces(pos: Vec3, inv_dir: Vec3, min: Vec3, max: Vec3, face: &mut u32) -> Vec2 {
    let mut tmin = 0f32;
    let mut tmax = 1000000f32;

    for d in 0..3 {
        let t1 = (min[d] - pos[d]) * inv_dir[d];
        let t2 = (max[d] - pos[d]) * inv_dir[d];

        let a1 = f32::min(t1, t2);
        let a2 = f32::max(t1, t2);
        if a1 > tmin {
            tmin = a1;
        }

        if a2 < tmax {
            *face = d as u32;
            tmax = a2;
        }
    }

	return vec2(tmin, tmax);
}

pub fn intersection(pos: Vec3, inv_dir: Vec3, min: Vec3, max: Vec3) -> Vec2 {
    let mut a = 0u32;
    intersection_faces(pos, inv_dir, min, max, &mut a)
}

pub fn box_normal(side: u32, ray_dir: Vec3) -> Vec3 {
    let sign = ray_dir.signum();
    let sides = [vec3(sign.x, 0.0, 0.0), vec3(0.0, sign.y, 0.0), vec3(0.0, 0.0, sign.z)];
    sides[side as usize]
}