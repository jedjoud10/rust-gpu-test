use shared::*;
use spirv_std::{arch::*, image::Image, memory::{Scope, Semantics}};

pub struct Voxel {
    pub active: bool,
    pub reflective: bool,
    pub refractive: bool,
}

fn remap(pos: Vec3, scaling: f32) -> UVec3 {
    let y = pos.y.max(0.0) as u32;
    let mut temp = pos.floor().rem_euclid(Vec3::ONE * CHUNK_SIZE as f32 / scaling).as_uvec3();
    temp.y = y;
    temp
}

pub fn get(
    image: &Image!(3D, format=r8ui, sampled=false, depth=false),
    pos: Vec3,
    level: u32,
) -> Voxel {
    let scaling = 2.0f32.pow(level as f32);
    let pos = remap(pos, scaling);
    let bits = image.read(pos);

    Voxel {
        active: bits & 1 != 0,
        reflective: bits & 2 != 0,
        refractive: bits & 4 != 0,
    }
}

pub const fn neighbor_pos_to_index(mut pos: IVec3) -> u32 {
    pos.x += 1;
    pos.y += 1;
    pos.z += 1;
    (pos.x + pos.z * 3 + pos.y * 3 * 3) as u32
}

pub const fn neighbor_index_to_pos(index: u32) -> IVec3 {
    // N(ABC) -> N(A) x N(BC)
    let y = index / (3 * 3);   // x in N(A)
    let w = index % (3 * 3);  // w in N(BC)

    // N(BC) -> N(B) x N(C)
    let z = w / 3;        // y in N(B)
    let x = index % 3;        // z in N(C)
    let mut a = ivec3(x as i32, y as i32, z as i32);
    a.x -= 1;
    a.y -= 1;
    a.z -= 1;
    a
}

/*
pub fn get_neighbor_active(
    image: &Image!(3D, format=r8ui, sampled=false, depth=false),
    pos: Vec3,
) -> u32 {
    let pos = remap(pos);
    let mut out = 0u32;

    for i in 0..27 {
        let offset = neighbor_index_to_pos(i);
        out |= (image.read((pos.as_ivec3() + offset).as_uvec3()) as u32 & 1) << i;
    }

    /*
    let nx = image.read(pos + uvec3(1, 0, 0)) as u32 & 1;
    let ny = (image.read(pos + uvec3(0, 1, 0)) as u32 & 1) << 1;
    let nz = (image.read(pos + uvec3(0, 0, 1)) as u32 & 1) << 2;
    let nnx = (image.read(pos - uvec3(1, 0, 0)) as u32 & 1) << 3;
    let nny = (image.read(pos - uvec3(0, 1, 0)) as u32 & 1) << 4;
    let nnz = (image.read(pos - uvec3(0, 0, 1)) as u32 & 1) << 5;
    nx | ny | nz | nnx | nny | nnz
    */

    out
}
*/

fn indeed(params: &GenerationParams, pos: Vec3) -> Voxel {
    let mut sum = pos.y - 40f32;
    //sum += rng::hash13(pos) * 2f32;
    //sum += f32::sin(pos.x * 0.1) * 2f32;

    sum += noise::fbm_simplex_2d(pos.xz() * 0.02, 4, 0.4, 2.0) * 2.8;
    
    if rng::hash12(pos.xz()) * 70.0 > pos.y && rng::hash12(pos.xz() * 0.54) > 0.98 {
        //sum -= 30.0;
    }

    Voxel {
        active: sum < 0f32,
        //reflective: rng::hash13(pos) > 0.95,
        //refractive: rng::hash13(pos * 0.5849) > 0.35,
        reflective: false,
        refractive: false,
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
        "%_ptr_Image_int = OpTypePointer Image %i32",
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

#[spirv(compute(threads(2, 2, 2)))]
pub unsafe fn propagate(
    #[spirv(global_invocation_id)] gid: UVec3,
    #[spirv(local_invocation_id)] lid: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] src: &Image!(3D, format=r8ui, sampled=false, depth=false),
    #[spirv(descriptor_set = 0, binding = 1)] dst: &Image!(3D, format=r8ui, sampled=false, depth=false),
    #[spirv(workgroup)] mut workgroup: &mut u32,
) {
    // src is the higher resolution mip
    // dst is the lower resolution mip
    // every invocation is executed for the texels of src
    // this doesn't handle bounds propagation yet, only sparse stuff
    let src_texel = src.read(gid.xyz());
    let dst_texel_pos = gid.xyz() / 2;
    let lid_texel = lid.xyz();

    // init
    if lid_texel == UVec3::ZERO {
        *workgroup = 0;
    }

    workgroup_memory_barrier_with_group_sync();
    
    // atomic stuff
    const SCOPE: u32 = Scope::Workgroup as u32;
    const SEMANTICS: u32 =  Semantics::WORKGROUP_MEMORY.bits() as u32;
    atomic_or::<u32, SCOPE, SEMANTICS>(&mut workgroup, src_texel);
    workgroup_memory_barrier_with_group_sync();

    // write to dst
    if lid_texel == UVec3::ZERO {
        let val = atomic_load::<u32, SCOPE, SEMANTICS>(&workgroup);
        dst.write(dst_texel_pos, uvec4(val, 0, 0, 0));
    }
}
