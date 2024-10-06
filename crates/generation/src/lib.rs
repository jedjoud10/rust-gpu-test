#![no_std]
#![allow(unused_imports)]
#![feature(asm_experimental_arch)]
#![cfg_attr(target_arch = "spirv", no_std)]

use shared::*;

fn indeed(pos: Vec3) -> bool {
    let mut sum = 0f32;

    for i in 1..5 {
        let scale = f32::powf(2f32, i as f32);
        let amplitude = f32::powf(0.5f32, i as f32);
        sum += f32::sin((pos.x + pos.z * 1.2) * 0.40f32 * scale) * amplitude * 6f32;
    }
    
    sum > pos.y
}


#[spirv(compute(threads(8, 8, 8)))]
pub unsafe fn main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image!(3D, format=r8ui, sampled=false, depth=false),
) {
    let test = indeed(id.xyz().as_vec3());
    image.write(id.xyz(), UVec4::from((if test { 1 } else { 0 }, 0, 0, 0)));
}