use shared::*;
use spirv_std::RuntimeArray;
use crate::{lighting::{self, light}, voxel};

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

    let raymarch = raymarch_internal(constants.position.xyz(), dir, mips);
    let mut lighting = raymarch.output;
    /*
    match DebugRenderMode::from(constants.mode) {
        DebugRenderMode::Default => {
            lighting /= f32::powf(2f32, f32::max(raymarch.reflections as f32 - 1.0, 0.0));
            lighting *= raymarch.refraction_tint;
        },
        DebugRenderMode::Iteration => {
            lighting = Vec3::ONE * raymarch.iteration_percent;
        },
        _ => panic!(),
    }
    */


    lighting *= raymarch.reflection_tint;
    lighting *= raymarch.refraction_tint;    
    //lighting = Vec3::lerp(lighting, Vec3::ONE, (raymarch.fog_sum * 0.01).clamp(0.0, 1.0));
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
}

fn box_normal(side: u32, sign: Vec3) -> Vec3 {
    let sides = [vec3(sign.x, 0.0, 0.0), vec3(0.0, sign.y, 0.0), vec3(0.0, 0.0, sign.z)];
    sides[side as usize]
}


pub const STEPS: u32 = 256;
pub const MAX_REFLECTIONS: u32 = 8;
pub const MAX_REFRACTIONS: u32 = 8;

// https://www.shadertoy.com/view/lfyGRW
pub fn raymarch_internal(
    ray_start: Vec3,
    mut ray_dir: Vec3,
    image: &[Image!(3D, format=r8ui, sampled=false, depth=false); MAX_MIPS as usize],
) -> RaymarchOutput2 {
    let mut starting_bozo = ray_start;
    let mut pos = starting_bozo.floor();
    let mut sign = ray_dir.signum();
    let mut inv_dir = ray_dir.recip();
    let mut side_dist = (pos - starting_bozo + 0.5 + 0.5 * sign) * inv_dir; 
    let mut face = 0;
    let mut reflections = 1;
    let mut refractions = 1;


    let mut refraction_tint = Vec3::ONE;
    let mut reflection_tint = Vec3::ONE;

    for x in 0..STEPS  {
        // Early break
        if pos.cmplt(Vec3::ZERO).any() || pos.cmpgt(Vec3::ONE * CHUNK_SIZE as f32).any()  {
            break;
        }

        /*
        for u in 1..3 {
            let i = 3-u;
            let k = i*3;
            let scaling = 2.0f32.pow(k as f32);
            let mut temppos = (starting_bozo / scaling).floor();
            let mut tempsidedists = (temppos * scaling - starting_bozo + scaling * 0.5 + scaling * 0.5 * sign) * inv_dir;
            let last = voxel::get(&image[k], temppos, k as u32).active;
            if !last {

                if tempsidedists.x < tempsidedists.y && tempsidedists.x < tempsidedists.z {
                    temppos.x += sign.x;
                    tempsidedists.x += sign.x * inv_dir.x * scaling;
                } else if tempsidedists.y < tempsidedists.z {
                    temppos.y += sign.y;
                    tempsidedists.y += sign.y * inv_dir.y * scaling; 
                } else {
                    temppos.z += sign.z;
                    tempsidedists.z += sign.z * inv_dir.z * scaling;  
                }

                let new = voxel::get(&image[k], temppos, k as u32).active;



                if !new {
                    let test = (temppos * scaling - starting_bozo + scaling * 0.5 - scaling * 0.5 * sign) * inv_dir; 
                    let max = test.max_element();
                    let world = starting_bozo + ray_dir * max;

                    let copy = world;
                    //pos = copy.floor();
                    //side_dist = (pos - copy + 0.5 + 0.5 * sign) * inv_dir; 
                    break;
                }
            } else {
                //break;
            }
        }
        */

        // Literally stolen from that shadertoy link to handle UV coords. Thankies DapperCore
        // This first calculates world position, and then subtracts pos to calculate local position
        let test = (pos - starting_bozo + 0.5 - 0.5 * sign) * inv_dir; 
        let max = test.max_element();
        let world = starting_bozo + ray_dir * max;

        // Voxel bitmask shenanigans
        let voxel = voxel::get(&image[0], pos, 0);
        let voxel_type = voxel::VOXEL_TYPES[voxel.id as usize];
        if voxel.active {
            let local_unshifted = world - pos;

            // we shift the local pos slightly inwards so that we avoid floating point precision errors 
            let spherical_normal = (local_unshifted - 0.5).normalize();
            let local = local_unshifted - spherical_normal * 0.01f32;

            let local_pixelated = local.div_euclid(Vec3::ONE / 8.0);
            let normal = -box_normal(face, sign);
            let mut should_continue = false;



            // Case where we modify teh ray direction
            if voxel_type.reflective || voxel_type.refractive {
                // if normal offset isnt zero then we must sample multiple rays
                // if reflection:
                //   initiate multiple rays with different ray dirs
                //   halt the current ray stuff (store the temp data somewhere)
                //   initiate the other rays
                //   when other rays are done (and their rays, and their rays)
                //   take the average lighting values
                //   average out their values eventually
                // ACTUALLY NO!!!
                // we can just do the shiddy TAA / accumulation method and reproject stuff instead!!!
                
                let normal_offset = (rng::hash33(world * vec3(42.594, 12.435, 65.945)) - 0.5) * 0.2f32;
                //let normal_offset = Vec3::ZERO;

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

            // Actual end case where we output the voxel values
            if !should_continue {
                //let combined = voxel::get_neighbor_active(image, pos);
                let combined = u32::MAX;

                /*
                return RaymarchOutput {
                    block_pos: pos.floor(),
                    ray_start: starting_bozo,
                    normal,
                    spherical_normal, 
                    reflections,
                    position: world,
                    hit: true,
                    neighbors_bitwise: combined,
                    refraction_tint,
                    iteration_percent: x as f32 / STEPS as f32,
                    ray_dir,
                    local,
                    local_pixelated,
                };
                */

                let test = lighting::LightingFnParams {
                    pos,
                    local_pixelated,
                    normal,
                    voxel: voxel_type,
                };

                return RaymarchOutput2 {
                    output: lighting::light(test),
                    refraction_tint,
                    reflection_tint,
                };
            }
        }

        /*
        let last = voxel::get(&image[k], temppos, k as u32).active;
            if !last {

                if tempsidedists.x < tempsidedists.y && tempsidedists.x < tempsidedists.z {
                    temppos.x += sign.x;
                    tempsidedists.x += sign.x * inv_dir.x * scaling;
                } else if tempsidedists.y < tempsidedists.z {
                    temppos.y += sign.y;
                    tempsidedists.y += sign.y * inv_dir.y * scaling; 
                } else {
                    temppos.z += sign.z;
                    tempsidedists.z += sign.z * inv_dir.z * scaling;  
                }

                let new = voxel::get(&image[k], temppos, k as u32).active;



                if !new {
                    let test = (temppos * scaling - starting_bozo + scaling * 0.5 - scaling * 0.5 * sign) * inv_dir; 
                    let max = test.max_element();
                    let world = starting_bozo + ray_dir * max;

                    let copy = world;
                    //pos = copy.floor();
                    //side_dist = (pos - copy + 0.5 + 0.5 * sign) * inv_dir; 
                    break;
                }
            } else {
                //break;
            }

        
        for i in 0..64 {
        }
        */
        
        increment_side_dist(&mut side_dist, &mut pos, sign, inv_dir, &mut face);
    }

    return RaymarchOutput2 {
        output: lighting::sky(starting_bozo, ray_dir),
        refraction_tint,
        reflection_tint,
    };
}

// Ok so I feel like I'm on the very edge of grasping *why* we can do this but not really. Something isn't clicking in my brain but who cares it works!!! (defo not stolen from gpt)
#[inline]
fn increment_side_dist(side_dist: &mut Vec3, pos: &mut Vec3, sign: Vec3, inv_dir: Vec3, face: &mut u32) {
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
}