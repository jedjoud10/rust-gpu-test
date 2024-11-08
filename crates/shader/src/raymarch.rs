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
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image!(2D, format=rgba8_snorm, sampled=false, depth=false),
    #[spirv(descriptor_set = 0, binding = 1)] mips: &[Image!(3D, format=r8ui, sampled=false, depth=false); MAX_MIPS as usize],
    #[spirv(uniform, descriptor_set = 0, binding = 2)] constants: &RaymarchParams,
) {
    let mut coords = Vec2::new(id.x as f32 / constants.width, id.y as f32 / constants.height);
    coords -= 0.5f32;
    coords *= 2.0f32;
    coords.y = -coords.y;

    let mut _dir = constants.proj_matrix.inverse().mul_vec4(vec4(coords.x, coords.y, -1f32, 1f32));
    _dir.w = 0f32;
    let dir = constants.view_matrix.inverse().mul_vec4(_dir).xyz().normalize();
    let mut lighting = Vec3::ONE;
    
    
    let ray_start = constants.position.xyz();
    let ray_dir = dir;
    let max_level: i32 = MAX_MIPS as i32 - 2;
    let mut level = max_level;
    let mut world = ray_start;

    // base: start level 0
    let mut count = 0;
    let mut min = max_level;
    let mut oob = false;
    let mut sky = false;
    let mut sum = 0u32;
    let mut face = 0u32;

    let sign = ray_dir.signum();
    let inv_dir = ray_dir.recip();

    // how many octree level changes we can do in a ray
    while count < 16 {
        if level < 0 {
            break;
        }
        
        if recursive_octree_3d_dda::<256>(world, ray_dir, sign, inv_dir, level as u32, mips, &mut world, &mut oob, &mut sum, &mut face) {
            // if hit something, continue to level n-1 (higher res)
            level -= 1;

            if level < min && level >= 0 {
                min = level;
            }
        } else {
            // if level n-1 misses, go back to level n (lower res)
            level += 1;

            /*
            // if position at level n is empty, go to n-1 recursively (which we then assume to be n)
            for _ in 0..32 {
                let divisor = 2u32.pow(level as u32) as f32;
                if !voxel::get(&mips[level as usize], world / divisor, level as u32).active {
                    level+= 1;
                } else {
                    break;
                }
            }
            */
        }

        if oob {
            sky = true;
            break;
        }

        count += 1; 
    }

    let normal = -box_normal(face, sign);
    lighting = lighting::light(world, world % Vec3::ONE, normal);

    if sky {
        lighting = lighting::skybox(ray_start, ray_dir);
    }
    
    //lighting *= ;
    //lighting *= ;
    //lighting *= ;

    match DebugRenderMode::from(constants.mode) {
        DebugRenderMode::Default => {},
        DebugRenderMode::Iteration => {
            lighting = Vec3::ONE * count as f32 / 32.0f32;
        },
        DebugRenderMode::Iteration1 => {
            lighting = Vec3::ONE * sum as f32 / 32.0f32;
        },
        DebugRenderMode::Iteration2 => {
            lighting = Vec3::ONE * min as f32 / max_level as f32;
        },
        DebugRenderMode::Normal => {
            lighting = normal;
        },
    }
    
    
    /*
    let raymarch = raymarch_internal(ray_start /* + dir * world.distance(ray_start) * 0.99 */, dir, mips);
    lighting = raymarch.output;
    match DebugRenderMode::from(constants.mode) {
        DebugRenderMode::Default => {
            lighting *= raymarch.reflection_tint;
            lighting *= raymarch.refraction_tint; 
        },
        DebugRenderMode::Iteration => {
            lighting = Vec3::ONE * raymarch.iteration_percent;
        },
        _ => panic!(),
    }
    */
    //lighting = Vec3::lerp(lighting, Vec3::ONE, (raymarch.fog_sum * 0.01).clamp(0.0, 1.0));
    //lighting = output;
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
    pub iteration_percent: f32,
}

fn box_normal(side: u32, sign: Vec3) -> Vec3 {
    let sides = [vec3(sign.x, 0.0, 0.0), vec3(0.0, sign.y, 0.0), vec3(0.0, 0.0, sign.z)];
    sides[side as usize]
}


pub const STEPS: u32 = 64;
pub const MAX_REFLECTIONS: u32 = 3;
pub const MAX_REFRACTIONS: u32 = 3;

// use the dda algorithm on specifically one level of the octree
pub fn recursive_octree_3d_dda<const ITERS: usize>(
    ray_start: Vec3,
    ray_dir: Vec3,
    sign: Vec3,
    inv_dir: Vec3,
    level: u32,
    image: &[Image!(3D, format=r8ui, sampled=false, depth=false); MAX_MIPS as usize],
    world: &mut Vec3,
    oob: &mut bool,
    sum: &mut u32,
    face: &mut u32,
) -> bool {
    let divisor = 2u32.pow(level) as f32;
    let mut pos = (ray_start / divisor).floor() * divisor;
    let mut side_dist = (pos - ray_start + divisor * 0.5 + divisor * 0.5 * sign) * inv_dir; 

    let mut x = 0;
    while x < ITERS  {
        if pos.cmplt(Vec3::ZERO).any() || pos.cmpge(Vec3::ONE * CHUNK_SIZE as f32).any()  {
            *oob = true;
            break;
        }

        // TODO: figure out how to avoid calculating this and just calculating side_dist and pos across octree boundaries
        let test = (pos - ray_start + divisor * 0.5 - divisor * 0.5 * sign) * inv_dir; 
        let max = test.max_element();
        *world = ray_start + ray_dir * max;

        if x == 0 {
            *world = ray_start;
        }

        if !voxel::get(&image[level as usize], pos / divisor, level).active {
            let a = side_dist.cmpeq(side_dist.min_element() * Vec3::ONE);
            let c = vec3(a.x as u32 as f32, a.y as u32 as f32, a.z as u32 as f32);

            increment_side_dist2(&mut side_dist, divisor, &mut pos, sign, inv_dir, face);
        } else {
            return true;
        }

        x += 1;
        *sum += 1;
    }

    return false;
}

// https://www.shadertoy.com/view/lfyGRW
pub fn raymarch_internal(
    ray_start: Vec3,
    mut ray_dir: Vec3,
    image: &[Image!(3D, format=r8ui, sampled=false, depth=false); MAX_MIPS as usize],
) -> RaymarchOutput2 {
    let mut starting_bozo = ray_start;
    let mut pos = starting_bozo.floor();
    let mut pos2 = (starting_bozo / 16.0).floor() * 16.0;
    let mut sign = ray_dir.signum();
    let mut inv_dir = ray_dir.recip();
    let mut side_dist = (pos - starting_bozo + 0.5 + 0.5 * sign); 
    let mut side_dist2 = (pos2 - starting_bozo + 16.0 * 0.5 + 16.0 * 0.5 * sign); 
    let mut face = 0;
    let mut reflections = 0;
    let mut refractions = 0;
    let mut last = Vec3::ZERO;


    let mut refraction_tint = Vec3::ONE;
    let mut reflection_tint = Vec3::ONE;

    let mut x = 0;
    while x < STEPS  {
        // Early break
        if pos.cmplt(Vec3::ZERO).any() || pos.cmpgt(Vec3::ONE * CHUNK_SIZE as f32).any() || pos2.cmplt(Vec3::ZERO).any() || pos2.cmpgt(Vec3::ONE * CHUNK_SIZE as f32).any() {
            break;
        }

        /*
        let test = (pos2 - starting_bozo + 16.0 * 0.5 - 16.0 * 0.5 * sign); 
        let max = test.max_element();
        let world = starting_bozo + ray_dir * max;
        */

        /*
        if !voxel::get(&image[4], pos2 / 16.0, 4).active {
            let a = (side_dist2 * inv_dir).cmpeq((side_dist2 * inv_dir).min_element() * Vec3::ONE);
            let c = vec3(a.x as u32 as f32, a.y as u32 as f32, a.z as u32 as f32);
        
            pos2 += 16.0 * sign * c;
            side_dist2 += 16.0 * sign * c;
            
            /*
            pos = world.floor();
            last = pos;
            side_dist = (pos - world + 0.5 + 0.5 * sign); 
            */
            

            //pos += 16.0 * sign * c;
            //side_dist += 16.0 * sign * c;
            //last = pos;
            //side_dist = side_dist2 / 16.0;
            //pos = pos2;
            //side_dist = side_dist2;
            
            x += 1;
            continue;
        } else {
            let test = (pos2 - starting_bozo + 0.5 - 0.5 * sign); 
            let max = test.max_element();
            let world = starting_bozo + ray_dir * max;

            return RaymarchOutput2 {
                output: world.normalize(),
                refraction_tint,
                reflection_tint,
                iteration_percent: x as f32 / STEPS as f32,
            };
        }

        continue;
        */
        // Literally stolen from that shadertoy link to handle UV coords. Thankies DapperCore
        // This first calculates world position, and then subtracts pos to calculate local position
        let test = (pos - starting_bozo + 0.5 - 0.5 * sign) * inv_dir; 
        let max = test.max_element();
        let world = starting_bozo + ray_dir * max;

        // Voxel bitmask shenanigans
        let voxel = voxel::get(&image[0], pos, 0);
        //let voxel_type = voxel::VOXEL_TYPES[voxel.id as usize];
        if voxel.active {
            let local_unshifted = world - pos;

            // we shift the local pos slightly inwards so that we avoid floating point precision errors 
            let spherical_normal = (local_unshifted - 0.5).normalize();
            let local = local_unshifted - spherical_normal * 0.01f32;
            let normal = -box_normal(face, sign);
            let mut should_continue = false;

            /*
            // Case where we modify teh ray direction
            if voxel_type.reflective(&test) || voxel_type.refractive() {                
                //let normal_offset = (rng::hash33(world * vec3(42.594, 12.435, 65.945)) - 0.5) * 0.2f32;
                let normal_offset = Vec3::ZERO;
                if voxel_type.reflective && reflections < MAX_REFLECTIONS {
                    let reflected = utils::reflect(ray_dir, normal);
                    ray_dir = reflected + normal_offset;
                    reflection_tint *= rng::hash33(pos.floor()).normalize();
                    reflections += 1;
                } else if voxel_type.refractive && refractions < MAX_REFRACTIONS {
                    ray_dir = utils::refract((world - starting_bozo).normalize(), normal + normal_offset, 1.0 / 1.5);
                    refraction_tint *= rng::hash33(pos.floor()).normalize();
                    refractions += 1;
                } 

                ray_dir = ray_dir.normalize();
                sign = ray_dir.signum();
                inv_dir = ray_dir.recip();
    
                let copy = world + ray_dir * 0.01;
                pos = copy.floor();
                
                starting_bozo = copy;
                side_dist = (pos - copy + 0.5 + 0.5 * sign) * inv_dir; 
                should_continue = true;
            }
            */

            // Actual end case where we output the voxel values
            if !should_continue {
                let mut temp = Vec3::ZERO;

                return RaymarchOutput2 {
                    output: lighting::light(pos, local, normal),
                    refraction_tint,
                    reflection_tint,
                    iteration_percent: x as f32 / STEPS as f32,
                };
            }
        }

        // single one indeed so tehe :3
        increment_side_dist(&mut side_dist, &mut pos, sign, inv_dir, &mut face);
        x += 1;
    }

    /*
    return RaymarchOutput2 {
        output: lighting::sky(starting_bozo, ray_dir),
        refraction_tint,
        reflection_tint,
        iteration_percent: x as f32 / STEPS as f32,
    };
    */

    return RaymarchOutput2 {
        output: skybox(starting_bozo, ray_dir),
        refraction_tint,
        reflection_tint,
        iteration_percent: x as f32 / STEPS as f32,
    };
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

    *pos += scale *sign * c;
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
fn increment_side_dist(side_dist: &mut Vec3, pos: &mut Vec3, sign: Vec3, inv_dir: Vec3, face: &mut u32) {
    let a = (*side_dist * inv_dir).cmpeq((*side_dist * inv_dir).min_element() * Vec3::ONE);
    let c = vec3(a.x as u32 as f32, a.y as u32 as f32, a.z as u32 as f32);

    *pos += sign * c;
    *side_dist += sign * c;

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