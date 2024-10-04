#![no_std]
#![allow(unused_imports)]
#![feature(asm_experimental_arch)]
#![cfg_attr(target_arch = "spirv", no_std)]

mod intersection;
mod raymarch;
mod blit;
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

fn dist(pos: Vec3) -> f32 {
    let floor = pos.y;
    let sphere = pos.length() - 1.0f32;

    floor.min(sphere)
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

    let mut ray_dir = constants.proj_matrix.inverse().mul_vec4(vec4(coords.x, coords.y, -1f32, 1f32));
    ray_dir.w = 0f32;
    let ray_dir = constants.view_matrix.inverse().mul_vec4(ray_dir).xyz().normalize();
    let inv_dir = ray_dir.recip();

    let mut color = Vec3::ZERO;
    let mut pos = constants.position.xyz() + ray_dir * 0.2f32;
    //color = ray_dir;

    for x in 0..64 {
        let min = (pos * 1f32).floor();
        let max = (pos * 1f32).ceil();

        let int = intersection(pos, inv_dir, min, max);
        let dist = f32::max(int.y, 0.001f32);
        pos += ray_dir * dist;

        let mut face = 0u32;
        intersection_faces(pos + ray_dir * 0.01f32, -inv_dir, min, max, &mut face);

        /* 
        if (f32::sin(min.x * 1.00f32) * 6.0f32 > min.y) {
            color = box_normal(face, -ray_dir);
            break;
        }
        */

        /*
        let dis = dist(pos);
        pos += ray_dir * dis;

        if dis < 0.001 {
            //color = pos;
            break;
        }
        */

        color.x = x as f32 / 64.0f32;
    }

    //image.write(id.xy(), vec4(coords.y, 0f32, 0f32, 1f32));
    image.write(id.xy(), Vec4::from((color, 1f32)));
}