use shared::*;
use spirv_std::{num_traits, RuntimeArray};
use crate::{lighting::{self, light, skybox}, voxel};

// ok so the main "octree" optimization can work in two ways
// 1) do a big "sparse" pass in the raymarch shader that will handle the larger octree chunks
// 2) implement sparse hopping directly in the raymarching algorithm
// the second option allows us to keep reflections and refractions as those could still make use of the octree hopping algo
// but the first one won't, since all it does is just get us to the scene as fast as possible, but that would be REALLY efficient for getting to the scene as fast as possible

#[spirv(compute(threads(32, 32, 1)))]
pub unsafe fn raymarch(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(local_invocation_id)] lid: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
    #[spirv(descriptor_set = 0, binding = 1)] mips: &[Image!(3D, format=r8ui, sampled=false, depth=false); MAX_MIPS as usize],
    #[spirv(workgroup)] var: &mut u32,
    #[spirv(uniform, descriptor_set = 0, binding = 2)] constants: &RaymarchParams,
) {
    if lid == UVec3::ZERO {
        *var = 0;
        spirv_std::arch::workgroup_memory_barrier();
    }

    let mut coords = Vec2::new(id.x as f32 / constants.width, id.y as f32 / constants.height);
    coords -= 0.5f32;
    coords *= 2.0f32;
    coords.y = -coords.y;

    let mut _dir = constants.proj_matrix.inverse().mul_vec4(vec4(coords.x, coords.y, -1f32, 1f32));
    _dir.w = 0f32;
    let dir = constants.view_matrix.inverse().mul_vec4(_dir).xyz().normalize();
    
    let ray_start = constants.position.xyz();
    let ray_dir = dir;
    
    let lighting = trace(ray_start, ray_dir, mips);
    /*
    let raymarch = raymarch_internal(ray_start, ray_dir,  mips);
    let mut lighting = raymarch.output;
    match DebugRenderMode::from(constants.mode) {
        DebugRenderMode::Default => {
            lighting *= raymarch.reflection_tint;
            lighting *= raymarch.refraction_tint; 
        },
        DebugRenderMode::Iteration => {
            if coords.x < 0.0 {
                lighting = Vec3::ONE * raymarch.iteration_percent.x;
            } else {
                lighting = Vec3::ONE * raymarch.iteration_percent.y;
            }
        },
        _ => lighting = vec3(0.58, 0.989, 0.24),
    }
    */
    
    image.write(id.xy(), Vec4::from((lighting, 1f32)));
}


#[derive(Default, Clone, Copy)]
pub struct RaymarchOutput {
    pub position: Vec3,
    pub local: Vec3,
    pub local_pixelated: Vec3,
    pub block_pos: Vec3,
    pub ray_dir: Vec3,
    pub ray_start: Vec3,
    pub neighbors_bitwise: u32,
    pub spherical_normal: Vec3,
    pub normal: Vec3,
    pub hit: bool,
    pub reflections: u32,
    pub refraction_tint: Vec3,
    pub iteration_percent: f32,
}

#[derive(Default, Clone, Copy)]
pub struct RaymarchOutput2 {
    pub output: Vec3,
    pub refraction_tint: Vec3,
    pub reflection_tint: Vec3,
    pub iteration_percent: Vec2,
}

fn box_normal(side: u32, sign: Vec3) -> Vec3 {
    let sides = [vec3(sign.x, 0.0, 0.0), vec3(0.0, sign.y, 0.0), vec3(0.0, 0.0, sign.z)];
    sides[side as usize]
}


pub const STEPS: u32 = 64;
pub const MAX_REFLECTIONS: u32 = 3;
pub const MAX_REFRACTIONS: u32 = 3;


fn test2(pos: Vec3) -> f32 {
    (pos.y - 20.0).min(pos.length() - 50.0)
}

// https://www.shadertoy.com/view/lfyGRW
pub fn raymarch_internal(
    ray_start: Vec3,
    ray_dir: Vec3,
    image: &[Image!(3D, format=r8ui, sampled=false, depth=false); MAX_MIPS as usize],
) -> RaymarchOutput2 {
    let starting_bozo = ray_start;
    let mut pos = starting_bozo.floor();
    let sign = ray_dir.signum();
    let inv_dir = ray_dir.recip();
    let mut side_dist = (pos - starting_bozo + 0.5 + 0.5 * sign); 
    let mut face = 0;
    let refraction_tint = Vec3::ONE;
    let reflection_tint = Vec3::ONE;

    let mut x = 0;
    while x < 256  {
        // Literally stolen from that shadertoy link to handle UV coords. Thankies DapperCore
        // This first calculates world position, and then subtracts pos to calculate local position
        let test = (pos - starting_bozo + 0.5 - 0.5 * sign) * inv_dir; 
        let max = test.max_element();
        let world = starting_bozo + ray_dir * max;

        // Voxel bitmask shenanigans
        //let voxel = voxel::get(&image[0], pos, 0);
        //let voxel_type = voxel::VOXEL_TYPES[voxel.id as usize];
        
        let voxel = crate::Voxel {
            active: test2(pos) < 0.0,
            id: 0,
        };
        
        if voxel.active {
            let local_unshifted = world - pos;

            // we shift the local pos slightly inwards so that we avoid floating point precision errors 
            let spherical_normal = (local_unshifted - 0.5).normalize();
            let local = local_unshifted - spherical_normal * 0.01f32;
            let normal = -box_normal(face, sign);

            return RaymarchOutput2 {
                output: lighting::light(pos, local, normal),
                refraction_tint,
                reflection_tint,
                iteration_percent: vec2(x as f32 / STEPS as f32, 0.0),
            };
        }

        // single one indeed so tehe :3
        increment_side_dist(&mut side_dist, &mut pos, sign, inv_dir, &mut face, ray_start);
        x += 1;
    }

    return RaymarchOutput2 {
        output: skybox(starting_bozo, ray_dir),
        refraction_tint,
        reflection_tint,
        iteration_percent: vec2(x as f32 / STEPS as f32, 0.0),
    };
}


// https://www.shadertoy.com/view/lfyGRW
pub fn raymarch_internal2(
    ray_start: Vec3,
    ray_dir: Vec3,
    image: &[Image!(3D, format=r8ui, sampled=false, depth=false); MAX_MIPS as usize],
) -> RaymarchOutput2 {
    let mut pos = ray_start;
    let mut old = pos;

    
    let mut x = 0;
    while x < 256  {
        //let distance = voxel::read_raw(&image[0], pos, 0) as f32;
        let distance = test2(pos+ray_dir);

        if distance < 0.01f32 {
            let mut output = raymarch_internal(pos - ray_dir * 2.0, ray_dir, image);
            output.iteration_percent.y = x as f32 / STEPS as f32;
            return output;

            /*
            return RaymarchOutput2 {
                output: Vec3::ONE,
                refraction_tint: Vec3::ONE,
                reflection_tint: Vec3::ONE,
                iteration_percent: vec2(0.0, x as f32 / STEPS as f32),
            };
            */
        }

        
        x += 1;
        pos += ray_dir * distance;
    }

    return RaymarchOutput2 {
        output: skybox(ray_start, ray_dir),
        refraction_tint: Vec3::ONE,
        reflection_tint: Vec3::ONE,
        iteration_percent: vec2(0.0, x as f32 / STEPS as f32),
    };
}

pub fn trace(
    ray_start: Vec3,
    ray_dir: Vec3,
    image: &[Image!(3D, format=r8ui, sampled=false, depth=false); MAX_MIPS as usize],
) -> Vec3 {
    return Vec3::ZERO;
}


use core::arch::asm;
#[inline]
pub unsafe fn touch_nation(
    value: u32,
) -> u32 {
    let mut out = 0u32;
    asm! {
        "%u32 = OpTypeInt 32 0",
        //"%ext = OpExtInstImport \"GLSL.std.450\"",
        "%value = OpLoad %u32 {value}",
        "%out = OpExtInst %u32 %1 FindILsb %value",
        "OpStore {out} %out",
        value = in(reg) &value,
        out = in(reg) &mut out,
    }
    out
}

#[inline]
fn increment_side_dist2(side_dist: &mut Vec3, scale: f32, pos: &mut Vec3, sign: Vec3, inv_dir: Vec3, face: &mut u32) {
    let a = side_dist.cmpeq(side_dist.min_element() * Vec3::ONE);
    let c = vec3(a.x as u32 as f32, a.y as u32 as f32, a.z as u32 as f32);





    *side_dist += scale * sign * c * inv_dir;

    if a.x {
        *face = 0;
    } else if a.y {
        *face = 1;
    } else {
        *face = 2;
    }
}

// Ok so I feel like I'm on the very edge of grasping *why* we can do this but not really. Something isn't clicking in my brain but who cares it works!!! (defo not stolen from gpt)
#[inline]
fn increment_side_dist(side_dist: &mut Vec3, pos: &mut Vec3, sign: Vec3, inv_dir: Vec3, face: &mut u32, cam_pos: Vec3) {
    let a = (*side_dist * inv_dir).cmpeq((*side_dist * inv_dir).min_element() * Vec3::ONE);
    let c = vec3(a.x as u32 as f32, a.y as u32 as f32, a.z as u32 as f32);

    *pos += sign * c;
    *side_dist += sign * c;
    
    /*
    if (cam_pos.distance(*pos) < 1.0) {
    } else {
        *pos += sign * c;
        *side_dist += sign * c;
    }
    */



    if a.x {
        *face = 0;
    } else if a.y {
        *face = 1;
    } else {
        *face = 2;
    }

    //*face = b.trailing_zeros();
    //*face = unsafe { touch_nation(b) };

    /*
    if side_dist.x < side_dist.y && side_dist.x < side_dist.z {
        pos.x += sign.x;
        side_dist.x += sign.x * inv_dir.x; 
        *face = 0;
    } else if side_dist.y < side_dist.z {
        pos.y += sign.y;
        side_dist.y += sign.y * inv_dir.y; 
        *face = 1;
    } else {
        pos.z += sign.z;
        side_dist.z += sign.z * inv_dir.z;  
        *face = 2;
    }
    */
}