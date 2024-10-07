use shared::*;
use crate::RaymarchOutput;

pub fn light(input: RaymarchOutput) -> Vec3 {
    if input.hit {
        //return input.position / 10.0f32;
        //return input.iteration_percent * Vec3::ONE;
        let mut col = Vec3::ZERO;
        
        if input.neighbors_bitwise & 2 == 0 && input.local_pos.y > 0.95 && (input.local_pos.xz() - 0.5).abs().cmplt(Vec2::ONE * 0.95).all() {
            col += vec3(6.0, 43.0, 5.0) / 255.0;
        }
        
        col += (noise::hash13(input.block_pos) * 0.2 + 0.8) * vec3(45.0, 46.0, 45.0) / 255.0;
        col
    } else {
        sky(input.ray_dir)
    }
}

fn sky(dir: Vec3) -> Vec3 {
    dir.y * Vec3::ONE
}