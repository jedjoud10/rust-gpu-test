use glam::{Mat4, Quat, Vec2, Vec3};
use winit::{event::ElementState, keyboard::{KeyCode, PhysicalKey}};

#[derive(Default)]
pub struct Movement {
    pub position: Vec3,
    pub rotation: Quat,
    pub proj_matrix: Mat4,
    pub view_matrix: Mat4,

    local_velocity: Vec2,
    summed_mouse: (f32, f32),
}

impl Movement {
    pub fn update(&mut self, ratio: f32, delta: f32) {


        self.proj_matrix = glam::Mat4::perspective_rh(horizontal_to_vertical(130f32, ratio), ratio, 0.001f32, 1000f32);

        let forward = self.rotation.mul_vec3(Vec3::Z);
        let right = self.rotation.mul_vec3(Vec3::X);
        let up = self.rotation.mul_vec3(Vec3::Y);
        self.view_matrix = glam::Mat4::look_at_rh(self.position, forward + self.position, up);

        let velocity = forward * self.local_velocity.y + right * self.local_velocity.x;
        self.position += velocity * delta * 3.0f32;
    }

    pub fn mouse_delta(&mut self, x: f32, y: f32) {
        self.summed_mouse.0 += x;
        self.summed_mouse.1 += y;

        let sens = 1.0f32;
        self.rotation = Quat::from_axis_angle(Vec3::Y, self.summed_mouse.0 * -0.003 * sens) * Quat::from_axis_angle(Vec3::X, self.summed_mouse.1 * 0.003 * sens);
    }

    pub fn key_pressed(&mut self, key: PhysicalKey, state: ElementState) {
        let val = match state {
            ElementState::Pressed => 1f32,
            ElementState::Released => 0f32,   
        };

        if let PhysicalKey::Code(code) = key {
            match code {
                KeyCode::KeyW => self.local_velocity.y = val,
                KeyCode::KeyS => self.local_velocity.y = -val,
                KeyCode::KeyA => self.local_velocity.x = val,
                KeyCode::KeyD => self.local_velocity.x = -val,
                _ => {}
            }
        }        
    }
}

pub fn horizontal_to_vertical(hfov: f32, ratio: f32) -> f32 {
    2.0 * ((hfov.to_radians() / 2.0).tan() * (1.0 / (ratio))).atan()
}