use shared::*;
use crate::RaymarchOutput;

pub fn light(input: RaymarchOutput) -> Vec3 {
    if input.hit {
        //input.iteration_percent * Vec3::ONE
        let mut col = Vec3::ZERO;
        if input.neighbors_bitwise & 2 == 0 && input.local_pos.y > 0.95 && (input.local_pos.xz() - 0.5).abs().cmplt(Vec2::ONE * 0.95).all() {
            col += vec3(6.0, 43.0, 5.0) / 255.0;
        }

        /*
        if input.local_pos.x < 0.01f32 && input.neighbors_bitwise & (1 << 3) == 0 {
            col = -Vec3::X;
        } else if input.local_pos.x > 0.99f32 && input.neighbors_bitwise & (1 << 1) == 0 {
            col = Vec3::X;
        } else if input.local_pos.y < 0.01f32 && input.neighbors_bitwise & (1 << 4) == 0 {
            col = -Vec3::Y;
        } else if input.local_pos.y > 0.99f32 && input.neighbors_bitwise & (1 << 2) == 0 {
            col = Vec3::Y;
        } else if input.local_pos.z < 0.01f32 && input.neighbors_bitwise & (1 << 5) == 0 {
            col = -Vec3::Z;
        } else if input.local_pos.z > 0.99f32 && input.neighbors_bitwise & (1 << 3) == 0 {
            col = Vec3::Z;
        }
        */

        /*
        if input.local_pos.x > 0.9999f32 && input.neighbors_bitwise & (1 << 0) == 0 {
            col = Vec3::X;
        } else if input.local_pos.y > 0.9999f32 && input.neighbors_bitwise & (1 << 1) == 0 {
            col = Vec3::Y;
        } else if input.local_pos.z > 0.9999f32 && input.neighbors_bitwise & (1 << 2) == 0 {
            col = Vec3::Z;
        }
        */

        col += (noise::hash13(input.block_pos) * 0.2 + 0.8) * vec3(45.0, 46.0, 45.0) / 255.0;
        col
    } else {
        sky(input.ray_dir)
    }
}

fn sky(dir: Vec3) -> Vec3 {
    dir.y * Vec3::ONE
}