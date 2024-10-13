use shared::*;
use spirv_std::RuntimeArray;
use crate::{lighting::{self, light}, voxel};

#[spirv(compute(threads(8, 8, 1)))]
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
    
    let mut lighting = if raymarch.hit {
        lighting::light(raymarch)
    } else {
        lighting::sky(raymarch)
    };

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

fn box_normal(side: u32, ray_dir: Vec3) -> Vec3 {
    let sign = ray_dir.signum();
    let sides = [vec3(sign.x, 0.0, 0.0), vec3(0.0, sign.y, 0.0), vec3(0.0, 0.0, sign.z)];
    sides[side as usize]
}

// gpted from glsl def
fn refract(incident: Vec3, normal: Vec3, eta: f32) -> Vec3 {
    let dot_n_i = normal.dot(incident);
    let k = 1.0 - eta * eta * (1.0 - dot_n_i * dot_n_i);

    if k < 0.0 {
        // Total internal reflection: no refraction
        Vec3::ZERO
    } else {
        // Compute the refracted vector
        eta * incident - (eta * dot_n_i + k.sqrt()) * normal
    }
}

// found online here 
// https://math.stackexchange.com/questions/13261/how-to-get-a-reflection-vector
fn reflect(ray_dir: Vec3, normal: Vec3) -> Vec3 {
    ray_dir - 2f32 * (ray_dir.dot(normal)) * normal
}

pub const STEPS: u32 = 128;
pub const MAX_REFLECTIONS: u32 = 8;
pub const MAX_REFRACTIONS: u32 = 8;

// https://www.shadertoy.com/view/lfyGRW
pub fn raymarch_internal(
    ray_start: Vec3,
    mut ray_dir: Vec3,
    image: &[Image!(3D, format=r8ui, sampled=false, depth=false); MAX_MIPS as usize],
) -> RaymarchOutput {
    let mut starting_bozo = ray_start;
    let mut pos = starting_bozo.floor();
    let mut pos2 = (starting_bozo / 8.0).floor();
    let mut sign = ray_dir.signum();
    let mut inv_dir = ray_dir.recip();
    //let mut inv_dir2 = (ray_dir*8.0).recip();
    let mut side_dist = (pos - starting_bozo + 0.5 + 0.5 * sign) * inv_dir; 
    let mut side_dist2 = (pos2 * 8.0 - starting_bozo + 4.0 + 4.0 * sign) * inv_dir; 
    let mut face = 0;
    let mut reflections = 1;
    let mut refractions = 1;
    let mut refraction_tint = Vec3::ONE;

    for x in 0..STEPS  {
        // Early break
        if pos.y >= 90.0 {
            break;
        }

        /*
        for i in (0..1).rev() {
            let scaling = 2.0f32.pow(i as f32);
            let mut pos = starting_bozo.floor();
            let mut temp_side_dist = (pos * scaling - starting_bozo + 0.5 + 0.5 * sign) * inv_dir; 
        }
        */

        // Literally stolen from that shadertoy link to handle UV coords. Thankies DapperCore
        // This first calculates world position, and then subtracts pos to calculate local position
        let test = (pos - starting_bozo + 0.5 - 0.5 * sign) * inv_dir; 
        let max = test.max_element();
        let world = starting_bozo + ray_dir * max;

        // Voxel bitmask shenanigans
        let voxel = voxel::get(&image[0], pos, 0);
        if voxel.active {
            let local_unshifted = world - pos;

            // we shift the local pos slightly inwards so that we avoid floating point precision errors 
            let spherical_normal = (local_unshifted - 0.5).normalize();
            let local = local_unshifted - spherical_normal * 0.01f32;

            let local_pixelated = local.div_euclid(Vec3::ONE / 8.0);
            let normal = -box_normal(face, ray_dir);
            let mut should_continue = false;

            // Case where we modify teh ray direction
            if voxel.reflective || voxel.refractive {
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
                
                //let normal_offset = (rng::hash33(world * vec3(42.594, 12.435, 65.945)) - 0.5) * 0.4f32;
                let normal_offset = Vec3::ZERO;

                if voxel.reflective && reflections < MAX_REFLECTIONS {
                    let reflected = reflect(ray_dir, normal);
                    ray_dir = reflected + normal_offset;
                    reflections += 1;
                } else if voxel.refractive && refractions < MAX_REFRACTIONS {
                    ray_dir = refract((world - starting_bozo).normalize(), normal + normal_offset, 1.0 / 1.5);
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
            }
        }
        
        let last = voxel::get(&image[3], pos2, 3).active;
        if !last {

            if side_dist2.x < side_dist2.y && side_dist2.x < side_dist2.z {
                pos2.x += sign.x;
                side_dist2.x += sign.x * inv_dir.x * 8.0;
            } else if side_dist2.y < side_dist2.z {
                pos2.y += sign.y;
                side_dist2.y += sign.y * inv_dir.y * 8.0; 
            } else {
                pos2.z += sign.z;
                side_dist2.z += sign.z * inv_dir.z * 8.0;  
            }

            let new = voxel::get(&image[3], pos2, 3).active;

            

            if last == new && !new {
                let test = (pos2 * 8.0 - starting_bozo + 4.0 - 4.0 * sign) * inv_dir; 
                let max = test.max_element();
                let world = starting_bozo + ray_dir * max;

                let copy = world + ray_dir * 0.01;
                pos = copy.floor();
                starting_bozo = copy;
                side_dist = (pos - copy + 0.5 + 0.5 * sign) * inv_dir; 
            }
        }

        // Ok so I feel like I'm on the very edge of grasping *why* we can do this but not really. Something isn't clicking in my brain but who cares it works!!! (defo not stolen from gpt)
        //let mut a = 1.0;
        if side_dist.x < side_dist.y && side_dist.x < side_dist.z {
            pos.x += sign.x;
            side_dist.x += sign.x * inv_dir.x; 
            face = 0;
        } else if side_dist.y < side_dist.z {
            pos.y += sign.y;
            side_dist.y += sign.y * inv_dir.y; 
            face = 1;
        } else {
            pos.z += sign.z;
            side_dist.z += sign.z * inv_dir.z;  
            face = 2;
        }
    }

    RaymarchOutput {
        ray_dir,
        ray_start: starting_bozo,
        reflections,
        refraction_tint,
        iteration_percent: 1.0,
        ..Default::default()    
    }
}