#![no_std]

pub use glam;
pub use spirv_std;
pub use crevice;
pub use bytemuck;
pub mod rng;
pub mod noise;
pub mod utils;

use crevice::std430::AsStd430;
pub use glam::*;
pub use bytemuck::{Pod, Zeroable};
pub use spirv_std::float::*;
pub use spirv_std::num_traits::*;
pub use spirv_std::number::*;
pub use spirv_std::num_traits::real::*;
pub use spirv_std::Image;
pub use spirv_std::{glam::UVec3, spirv};

pub const CHUNK_SIZE: u32 = 256;
pub const MAX_MIPS: u32 = CHUNK_SIZE.trailing_zeros() + 1;
pub const SIZE_REDUCTION: u32 = 1;

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum DebugRenderMode {
    Default=0,
    Normal=1,
    Iteration=2,
    Iteration1=3,
    Iteration2=4,
}

impl Into<u32> for DebugRenderMode {
    fn into(self) -> u32 {
        unsafe { core::mem::transmute(self) }
    }
}

impl From<u32> for DebugRenderMode {
    fn from(value: u32) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd430))]
pub struct RaymarchParams {
    pub proj_matrix: glam::Mat4,
    pub view_matrix: glam::Mat4,
    pub position: glam::Vec4,
    pub width: f32,
    pub height: f32,
    pub mode: u32,
}

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd430))]
pub struct LightingParams {
    pub light_dir: glam::Vec4,
    pub ambient_boost: f32,
}

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd430))]
pub struct GenerationParams {
    pub time: f32,
}