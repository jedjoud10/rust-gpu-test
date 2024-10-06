#![no_std]

pub use glam;
pub use spirv_std;
pub use crevice;
pub use bytemuck;

use crevice::std430::AsStd430;
pub use glam::*;
pub use bytemuck::{Pod, Zeroable};
pub use spirv_std::float::*;
pub use spirv_std::num_traits::*;
pub use spirv_std::number::*;
pub use spirv_std::num_traits::real::*;
pub use spirv_std::Image;
pub use spirv_std::{glam::UVec3, spirv};

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd430))]
pub struct RaymarchParams {
    pub proj_matrix: glam::Mat4,
    pub view_matrix: glam::Mat4,
    pub position: glam::Vec4,
    pub width: f32,
    pub height: f32,
}