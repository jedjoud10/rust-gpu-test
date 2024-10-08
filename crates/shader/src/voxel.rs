use shared::*;
use spirv_std::{image::Image, memory::{Scope, Semantics}};

pub struct Voxel {
    pub active: bool,
    pub reflective: bool,
    pub refractive: bool,
}

fn remap(pos: Vec3) -> UVec3 {
    let y = pos.y.max(0.0) as u32;
    let mut temp = pos.floor().rem_euclid(Vec3::ONE * CHUNK_SIZE as f32).as_uvec3();
    temp.y = y;
    temp
}

pub fn get(
    image: &Image!(3D, format=r8ui, sampled=false, depth=false),
    pos: Vec3,
) -> Voxel {
    let pos = remap(pos);
    let bits = image.read(pos);

    Voxel {
        active: bits & 1 != 0,
        reflective: bits & 2 != 0,
        refractive: bits & 4 != 0,
    }
}

pub fn get_neighbor_active(
    image: &Image!(3D, format=r8ui, sampled=false, depth=false),
    pos: Vec3,
) -> u32 {
    let pos = remap(pos);
    let nx = image.read(pos + uvec3(1, 0, 0)) as u32 & 1;
    let ny = (image.read(pos + uvec3(0, 1, 0)) as u32 & 1) << 1;
    let nz = (image.read(pos + uvec3(0, 0, 1)) as u32 & 1) << 2;
    let nnx = (image.read(pos - uvec3(1, 0, 0)) as u32 & 1) << 3;
    let nny = (image.read(pos - uvec3(0, 1, 0)) as u32 & 1) << 4;
    let nnz = (image.read(pos - uvec3(0, 0, 1)) as u32 & 1) << 5;
    nx | ny | nz | nnx | nny | nnz
}

fn indeed(params: &GenerationParams, pos: Vec3) -> Voxel {
    let mut sum = pos.y - 40f32;
    //sum += rng::hash13(pos) * 2f32;
    //sum += f32::sin(pos.x * 0.1) * 2f32;

    sum += noise::fbm_simplex_2d(pos.xz() * 0.02, 3, 0.5, 2.0) * 2.8;
    
    if rng::hash12(pos.xz()) * 70.0 > pos.y && rng::hash12(pos.xz() * 0.54) > 0.98 {
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
    #[spirv(push_constant)] constants: &GenerationParams,
) {
    let voxel = indeed(constants, id.xyz().as_vec3());
    let active = voxel.active as u32;
    let reflective = (voxel.reflective as u32) << 1;
    let refractive = (voxel.refractive as u32) << 2;
    let bitmask = active | reflective | refractive;

    image.write(id.xyz(), UVec4::from((bitmask, 0, 0, 0)));
}

#[spirv(compute(threads(8, 8, 8)))]
pub unsafe fn update(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] src: &Image!(3D, format=r8ui, sampled=false, depth=false),
    #[spirv(descriptor_set = 0, binding = 0)] dst: &Image!(3D, format=r8ui, sampled=false, depth=false),
) {
}

/*
use core::arch::asm;
#[inline]
pub unsafe fn atomic_or_ptr<const SCOPE: u32, const SEMANTICS: u32>(
    value: u32,
    image: &Image!(3D, format=r8ui, sampled=false, depth=false),
    texel: UVec3,
) {
    asm! {
        "%u32 = OpTypeInt 32 0",
        "%i32 = OpTypeInt 32 1",
        "%value = OpLoad _ {value}",
        "%x = OpLoad _ {x}",
        "%y = OpLoad _ {y}",
        "%z = OpLoad _ {z}",
        "%scope = OpConstant %u32 {scope}",
        "%semantics = OpConstant %u32 {semantics}",
        "%vec3u32 = OpTypeVector %u32 3",
        "%_ptr_Image_int = OpTypePointer Generic %i32",
        "%coord = OpCompositeConstruct %vec3u32 %x %y %z",
        "%ptr = OpImageTexelPointer %_ptr_Image_int {image} %coord %semantics",
        "%27 = OpAtomicOr %i32 %ptr %scope %semantics %value",
        scope = const SCOPE,
        semantics = const SEMANTICS,
        x = in(reg) &texel.x,
        y = in(reg) &texel.y,
        z = in(reg) &texel.z,
        image = in(reg) image,
        value = in(reg) &value,
    }
}
*/

#[spirv(compute(threads(8, 8, 8)))]
pub unsafe fn propagate(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] src: &Image!(3D, format=r8ui, sampled=false, depth=false),
    #[spirv(descriptor_set = 0, binding = 0)] dst: &Image!(3D, format=r8ui, sampled=false, depth=false),
) {
    // src is the higher resolution mip
    // dst is the lower resolution mip
    // every invocation is executed for the texels of src
    // this doesn't handle bounds propagation yet, only sparse stuff
    //let src_texel = src.read(id.xyz());
    let temp = 0u32;
    //atomic_or_ptr::<{ Scope::Workgroup as _ }, { Semantics::NONE.bits() as _ }>(temp, dst, id.xyz());
}
