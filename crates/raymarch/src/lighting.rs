use shared::*;
use crate::RaymarchOutput;

pub fn light(input: RaymarchOutput, dir: Vec3) -> Vec3 {
    if input.hit {
        input.normal
    } else {
        sky(dir)
    }
}

fn sky(dir: Vec3) -> Vec3 {
    dir
}