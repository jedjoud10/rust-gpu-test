use shared::*;
use spirv_std::{arch::{all_memory_barrier, memory_barrier, workgroup_memory_barrier_with_group_sync}, num_traits, RuntimeArray};
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
    #[spirv(uniform, descriptor_set = 0, binding = 2)] constants: &RaymarchParams,
) {
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
    image.write(id.xy(), Vec4::from((lighting, 1f32)));
}



// Need to separate it because rust-gpu does not support options yet
#[derive(Default, Clone, Copy)]
struct Intersection {
    hit: bool,
    normal: Vec3,
    position: Vec3,
}

#[derive(Default, Clone, Copy)]
struct Hit {
    normal: Vec3,
    position: Vec3,
}

// https://tavianator.com/2011/ray_box.html
// also gpted
fn intersection(box_min: Vec3, box_max: Vec3, ray_start: Vec3, inv_dir: Vec3) -> Intersection {
    let tmin = (box_min - ray_start) * inv_dir;
    let tmax = (box_max - ray_start) * inv_dir;

    let t1 = tmin.min(tmax);
    let t2 = tmin.max(tmax);

    let entry = t1.x.max(t1.y.max(t1.z));
    let exit = t2.x.min(t2.y.min(t2.z));

    if entry > exit || exit < 0.0 {
        return Intersection { hit: false, normal: Vec3::ZERO, position: Vec3::ZERO };
    }

    let a = t1.cmpeq(t1.max_element() * Vec3::ONE);
    let c = vec3(a.x as u32 as f32, a.y as u32 as f32, a.z as u32 as f32);

    //return None;
    return Intersection {
        hit: true,
        position: entry / inv_dir + ray_start,
        normal: c,
    };
}

#[derive(Default, Clone, Copy)]
struct PendingNode(u32);

// 3 first bytes used for pos, last byte used for level 

impl PendingNode {
    fn new(level: u32, node: UVec3) -> Self {
        Self(level | node.x << 8 | node.y << 16 | node.z << 24)
    }

    fn position(&self) -> UVec3 {
        uvec3((self.0 >> 8) & 0xFF, (self.0 >> 16) & 0xFF, (self.0 >> 24) & 0xFF)
    }

    fn center(&self) -> Vec3 {
        self.position().as_vec3() + (self.size() / 2) as f32 * Vec3::ONE
    }

    fn size(&self) -> u32 {
        1 << self.level()
    }

    fn level(&self) -> u32 {
        self.0 & 0xFF
    }
}

const OFFSETS: [UVec3; 8] = [
    uvec3(0, 0, 0),
    uvec3(0, 1, 0),
    uvec3(0, 0, 1),
    uvec3(0, 1, 1),
    uvec3(1, 0, 0),
    uvec3(1, 1, 0),
    uvec3(1, 0, 1),
    uvec3(1, 1,1),
];

use spirv_std::{arch::*, image::Image, memory::{Scope, Semantics}};

pub unsafe fn trace(
    ray_start: Vec3,
    ray_dir: Vec3,
    image: &[Image!(3D, format=r8ui, sampled=false, depth=false); MAX_MIPS as usize],
) -> Vec3 {
    let mut nodes = [PendingNode::default(); 16];
    
    // Add the root node
    let mut length = 1;
    let inv_dir = ray_dir.recip();
    nodes[0] = PendingNode::new(MAX_MIPS-1, UVec3::ZERO);
    
    for i in 0..64 {
        if length == 0 {
            return Vec3::ONE * i as f32 / 64.0;
        }

        // Sort the pending nodes by distance to the camera
        let mut closest_dist = 1000000.0;
        let mut closest_index = 100000;
        for x in 0..length {
            let temp2 = nodes[x];

            let dist = temp2.center().distance(ray_start);
            if dist < closest_dist {
                closest_dist = dist;
                closest_index = x;
            }
        }

        // Get the closest node to the camera and do a swap remove
        length -= 1;
        let temp = nodes[closest_index];
        nodes[closest_index] = nodes[length];

        // Exit when we reach the appropriate level
        if temp.level() == 4 {
            return Vec3::ONE * i as f32 / 64.0;
            //return Vec3::ONE * hits[closest_index].position / 100.0; 
            //return Vec3::ONE * closest_dist as f32 / 4000.0; 
        }

        // Octree traveral mome
        for x in 0..8 {
            let offset = OFFSETS[x];

            let level = temp.level() - 1;
            let position = temp.position() + offset * temp.size() / 2;

            let mut pending = PendingNode::new(level, position);

            let box_min = pending.position();
            let box_max = pending.position() + pending.size() * UVec3::ONE;
    
            let mapped = pending.center() / pending.size() as f32;
            let set = image[pending.level() as usize].read(mapped.floor().as_uvec3());
            let intersection = intersection(box_min.as_vec3(), box_max.as_vec3(), ray_start, inv_dir);

            // If the child contains volume itself, add it to the pending node list
            if set == 1 && intersection.hit {
                //pending.1 = intersection.normal;
                //hits[length] = Hit { normal: intersection.normal, position: intersection.position };
                nodes[length] = pending;

                length += 1;
            }
        
            if length >= 16 {
                return vec3(225.0, 52.0, 235.0) / 255.0;
            }
        }
    }
    
    return vec3(0.0, 1.0, 0.0);
}