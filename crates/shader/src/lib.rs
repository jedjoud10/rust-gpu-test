#![no_std]
#![allow(unused_imports)]
#![feature(asm_experimental_arch)]
#![cfg_attr(target_arch = "spirv", no_std)]

mod intersection;
mod raymarch;
mod blit;
mod lighting;
pub use blit::*;
pub use intersection::*;
pub use raymarch::*;

use glam::Vec3;
use glam::Vec3Swizzles;
use glam::Vec4;

use crate::intersection::*;
use bytemuck::{Pod, Zeroable};
use crevice::std430::AsStd430;
use glam::{vec2, vec4, Vec2, Vec4Swizzles};
use spirv_std::Image;
use spirv_std::float::*;
use spirv_std::num_traits::*;
use spirv_std::number::*;
use spirv_std::num_traits::float::*;
use spirv_std::num_traits::real::*;
use spirv_std::{glam::UVec3, spirv};


#[cfg_attr(not(target_arch = "spirv"), derive(AsStd430))]
pub struct ShaderConstants {
    pub proj_matrix: glam::Mat4,
    pub view_matrix: glam::Mat4,
    pub position: glam::Vec4,
    pub width: f32,
    pub height: f32,
}

#[spirv(compute(threads(32, 32, 1)))]
pub unsafe fn test(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
    #[spirv(uniform, descriptor_set = 0, binding = 1)] constants: &ShaderConstants,
) {
    let mut coords = Vec2::new(id.x as f32 / constants.width, id.y as f32 / constants.height);
    coords -= 0.5f32;
    coords *= 2.0f32;
    coords.y = -coords.y;

    let mut _dir = constants.proj_matrix.inverse().mul_vec4(vec4(coords.x, coords.y, -1f32, 1f32));
    _dir.w = 0f32;
    let dir = constants.view_matrix.inverse().mul_vec4(_dir).xyz().normalize();

    let output = raymarch::raymarch(constants.position.xyz(), dir);
    let output = lighting::light(output, dir);

    image.write(id.xy(), Vec4::from((output, 1f32)));
}