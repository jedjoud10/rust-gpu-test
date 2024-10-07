use shared::*;

pub struct Voxel {
    pub active: bool,
    pub reflective: bool,
    pub refractive: bool,
}

fn remap(pos: Vec3) -> UVec3 {
    pos.floor().abs().rem_euclid(Vec3::ONE * 128.0).as_uvec3()
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