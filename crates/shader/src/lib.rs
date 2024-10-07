#![no_std]
#![allow(unused_imports)]
#![feature(asm_experimental_arch)]
#![cfg_attr(target_arch = "spirv", no_std)]

use shared::*;
mod intersection;
mod raymarch;
mod lighting;
mod voxel;
mod blit;
pub use blit::*;
pub use intersection::*;
pub use raymarch::*;
pub use voxel::*;
use crate::intersection::*;

