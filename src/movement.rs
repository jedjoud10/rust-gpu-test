use shared::*;
use winit::{event::ElementState, keyboard::{KeyCode, PhysicalKey}};

use crate::input::{Axis, Input, MouseAxis};

#[derive(Default)]
pub struct Movement {
    pub position: Vec3,
    pub rotation: Quat,
    pub proj_matrix: Mat4,
    pub view_matrix: Mat4,

    local_velocity: Vec2,
    boost: f32,
}

impl Movement {
    pub fn update(&mut self, input: &Input, ratio: f32, delta: f32) {
        self.local_velocity = Vec2::ZERO;
        let speed = if input.get_button(KeyCode::ShiftLeft).held() {
            2f32 + 2f32.powf(self.boost)
        } else if input.get_button(KeyCode::ControlLeft).held() {
            0.25f32
        } else {
            1.0f32
        };
        
        if input.get_button(KeyCode::KeyW).held() {
            self.local_velocity.y = 1f32;
        } else if input.get_button(KeyCode::KeyS).held() {
            self.local_velocity.y = -1f32;
        }

        if input.get_button(KeyCode::KeyA).held() {
            self.local_velocity.x = 1f32;
        } else if input.get_button(KeyCode::KeyD).held() {
            self.local_velocity.x = -1f32;
        }


        self.boost += input.get_axis(Axis::Mouse(MouseAxis::ScrollDelta));
        self.boost = self.boost.clamp(0.0, 20.0);
        let sens = 1.0f32;
        let summed_mouse = (input.get_axis(Axis::Mouse(MouseAxis::PositionX)), input.get_axis(Axis::Mouse(MouseAxis::PositionY)));
        self.rotation = Quat::from_axis_angle(Vec3::Y, summed_mouse.0 * -0.003 * sens) * Quat::from_axis_angle(Vec3::X, summed_mouse.1 * 0.003 * sens);

        self.proj_matrix = Mat4::perspective_rh(horizontal_to_vertical(130f32, ratio), ratio, 0.001f32, 1000f32);

        let forward = self.rotation.mul_vec3(Vec3::Z);
        let right = self.rotation.mul_vec3(Vec3::X);
        let up = self.rotation.mul_vec3(Vec3::Y);
        self.view_matrix = Mat4::look_at_rh(self.position, forward + self.position, up);

        let velocity = forward * self.local_velocity.y + right * self.local_velocity.x;
        self.position += velocity * delta * 3.0f32 * speed;
    }
}

pub fn horizontal_to_vertical(hfov: f32, ratio: f32) -> f32 {
    2.0 * ((hfov.to_radians() / 2.0).tan() * (1.0 / (ratio))).atan()
}