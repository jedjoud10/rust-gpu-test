use glam::UVec2;
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

pub struct TestConstants {
    src_size: UVec2,
    dst_size: UVec2,
}

#[spirv(compute(threads(32, 32, 1)))]
pub unsafe fn blit(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] src: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
    #[spirv(descriptor_set = 0, binding = 1)] dst: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
    #[spirv(push_constant)] consts: &TestConstants
) {
}