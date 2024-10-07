#[allow(unused_imports)]
#[cfg_attr(target_arch = "spirv", no_std)]

use shared::*;

pub fn box_normal(side: u32, ray_dir: Vec3) -> Vec3 {
    let sign = ray_dir.signum();
    let sides = [vec3(sign.x, 0.0, 0.0), vec3(0.0, sign.y, 0.0), vec3(0.0, 0.0, sign.z)];
    sides[side as usize]
}