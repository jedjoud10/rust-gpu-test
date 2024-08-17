#![no_std]
#![allow(unused_imports)]
use bytemuck::{Pod, Zeroable};
use spirv_std::glam::{UVec2, UVec4, Vec3, Vec3Swizzles, Vec4};
use spirv_std::Image;
use spirv_std::float::*;
use spirv_std::num_traits::*;
use spirv_std::number::*;
use spirv_std::num_traits::float::*;
use spirv_std::num_traits::real::*;
use spirv_std::{glam::UVec3, spirv};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub time: f32,
}

#[spirv(compute(threads(32, 32, 1)))]
pub unsafe fn main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
) {
    let value = constants.time.sin() * 0.5f32 + 0.5f32;
    image.write(id.xy(), Vec4::new(value, 0f32, 0f32, 1f32));
}