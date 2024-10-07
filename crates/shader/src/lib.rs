#![no_std]
#![feature(asm_const)]
#![allow(unused_imports)]
#![feature(asm_experimental_arch)]
#![cfg_attr(target_arch = "spirv", no_std)]

use shared::*;
pub mod raymarch;
pub mod lighting;
pub mod voxel;
pub mod blit;
pub use blit::*;
pub use raymarch::*;
pub use voxel::*;
