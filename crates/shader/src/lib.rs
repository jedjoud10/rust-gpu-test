#![no_std]
#![allow(unused_imports)]
#![feature(asm_experimental_arch)]
#![cfg_attr(target_arch = "spirv", no_std)]

use shared::*;
mod intersection;
mod raymarch;
mod lighting;
mod voxel;
pub use intersection::*;
pub use raymarch::*;
use voxel::Voxel;
use crate::intersection::*;

#[spirv(compute(threads(32, 32, 1)))]
pub unsafe fn raymarch(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
    #[spirv(descriptor_set = 0, binding = 1)] texture: &Image!(3D, format=r8ui, sampled=false, depth=false),
    #[spirv(uniform, descriptor_set = 0, binding = 2)] constants: &RaymarchParams,
) {
    let mut coords = Vec2::new(id.x as f32 / constants.width, id.y as f32 / constants.height);
    coords -= 0.5f32;
    coords *= 2.0f32;
    coords.y = -coords.y;

    let mut _dir = constants.proj_matrix.inverse().mul_vec4(vec4(coords.x, coords.y, -1f32, 1f32));
    _dir.w = 0f32;
    let dir = constants.view_matrix.inverse().mul_vec4(_dir).xyz().normalize();

    let raymarch = raymarch::raymarch(constants.position.xyz(), dir, texture);
    
    let reflections = raymarch.reflections;
    let refraction = raymarch.refraction_tint;
    let mut output = if raymarch.hit {
        lighting::light(raymarch)
    } else {
        lighting::sky(raymarch.ray_start, raymarch.ray_dir)
    };

    output /= f32::powf(2f32, f32::max(reflections as f32 - 1.0, 0.0));
    output *= refraction;
    image.write(id.xy(), Vec4::from((output, 1f32)));
}

#[spirv(compute(threads(32, 32, 1)))]
pub unsafe fn blit(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] src: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
    #[spirv(descriptor_set = 0, binding = 1)] dst: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
    #[spirv(push_constant)] consts: &u32
) {
    let src_pos = id.xy() / *consts;
    let src_val: Vec4 = src.read(src_pos);
    dst.write(id.xy(), src_val);
}

fn indeed(pos: Vec3) -> Voxel {
    let mut sum = pos.y - 40f32;
    //sum += rng::hash13(pos) * 2f32;
    //sum += f32::sin(pos.x * 0.1) * 2f32;
    sum += noise::fbm_simplex_2d(pos.xz() * 0.02, 3, 0.5, 2.0) * 2.8;
    
    if rng::hash12(pos.xz()) * 60.0 > pos.y && rng::hash12(pos.xz() * 0.54) > 0.95 {
        sum -= 30.0;
    }

    Voxel {
        active: sum < 0f32,
        reflective: rng::hash13(pos) > 0.95,
        refractive: rng::hash13(pos * 0.5849) > 0.95,
    }
}


#[spirv(compute(threads(8, 8, 8)))]
pub unsafe fn generation(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image!(3D, format=r8ui, sampled=false, depth=false),
) {
    let voxel = indeed(id.xyz().as_vec3());
    let active = voxel.active as u32;
    let reflective = (voxel.reflective as u32) << 1;
    let refractive = (voxel.refractive as u32) << 2;
    let bitmask = active | reflective | refractive;

    image.write(id.xyz(), UVec4::from((bitmask, 0, 0, 0)));
}
