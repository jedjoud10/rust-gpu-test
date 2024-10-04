use glam::Vec3;

// TODO: Figure out to only have to define this once instead of multiple times
use spirv_std::float::*;
use spirv_std::num_traits::*;
use spirv_std::number::*;
use spirv_std::num_traits::float::*;
use spirv_std::num_traits::real::*;

use crate::RaymarchOutput;

pub fn light(input: RaymarchOutput, dir: Vec3) -> Vec3 {
    if input.hit {
        input.normal
    } else {
        sky(dir)
    }
}

fn sky(dir: Vec3) -> Vec3 {
    dir
}