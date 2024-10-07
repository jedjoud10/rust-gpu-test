use shared::*;

#[spirv(compute(threads(32, 32, 1)))]
pub unsafe fn blit(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] src: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
    #[spirv(descriptor_set = 0, binding = 1)] dst: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
) {
    let src_pos = id.xy() / SIZE_REDUCTION;
    let src_val: Vec4 = src.read(src_pos);
    dst.write(id.xy(), src_val);
}