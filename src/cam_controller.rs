use glam::Vec3;

use crate::{DeltaTime, physics::TCBody};

pub fn dir_from_angle(yaw: f32, pitch: f32) -> Vec3 {
    let (sin_pitch, cos_pitch) = pitch.sin_cos();
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize()
}

pub struct CamController {
    body: TCBody,
    pending_acc: Vec3,

    free_cam: bool,
    speed: f32, // camera speed

    yaw: f32, // source of truth
    pitch: f32,

    dir: Vec3,

    delta_time: DeltaTime,
}

impl CamController {
    const FRICTION: f32 = 1.0;
    const STANDART_SPEED: f32 = 20.0;
    const MAX_SPEED: f32 = 100.0;
    const ACC_CHANGE_SENSITIVITY: f32 = 3.0;
    const SENSITIVITY: f32 = 0.0025;

    pub fn new(pos: Vec3, yaw: f32, pitch: f32, free_cam: bool, delta_time: DeltaTime) -> Self {
        let dir = dir_from_angle(yaw, pitch);

        Self {
            body: TCBody::new(pos),
            pending_acc: Vec3::ZERO,

            free_cam,
            speed: Self::STANDART_SPEED,

            dir,
            yaw,
            pitch,

            delta_time,
        }
    }

    /// Dreht die Kamera um einen Winkel multipliziert mit der Kamera Sensitivität.
    pub fn rotate_around_angle(&mut self, yaw: f32, pitch: f32) {
        self.yaw += yaw * Self::SENSITIVITY;
        self.pitch += pitch * Self::SENSITIVITY;

        self.dir = dir_from_angle(self.yaw, self.pitch);
        // self.rot = Quat::from_rotation_y(yaw) * Quat::from_rotation_z(pitch);
    }

    /// Bewegt die Kamera in eine Richtung relativ zur Richtung in die die Kamera zeigt.
    pub fn add_input(&mut self, input_vector: Vec3) {
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let forward = dir_from_angle(self.yaw, self.pitch);
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        let impuls = forward * input_vector.x * self.speed
            + right * input_vector.z * self.speed
            + Vec3::Y * input_vector.y * self.speed;

        self.pending_acc = impuls;
    }

    pub fn add_acc(&mut self, acc: Vec3) {
        self.pending_acc = acc;
    }

    /// Takes a function which takes the current and the next position and returns the resolved position.
    pub fn advance_pos(&mut self, contrain: impl FnMut(Vec3, Vec3) -> Vec3) {
        self.body.step(self.delta_time(), Self::FRICTION);

        let dt = self.delta_time();

        self.body
            .constrain(|_, next_pos| next_pos + self.pending_acc * dt / 2. * dt);

        self.body.constrain(contrain);
    }

    pub fn update_speed(&mut self, change: f32) {
        self.speed *= (Self::ACC_CHANGE_SENSITIVITY * change).exp();

        self.speed = self.speed.clamp(
            Self::STANDART_SPEED / Self::MAX_SPEED,
            Self::STANDART_SPEED * Self::MAX_SPEED,
        );

        println!("new speed: {}", self.speed);
    }

    pub fn toggle_free_cam(&mut self) {
        self.free_cam = !self.free_cam
    }

    pub fn free_cam(&self) -> bool {
        self.free_cam
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time.get_f32()
    }

    pub fn pos(&self) -> Vec3 {
        self.body.pos()
    }

    pub fn dir(&self) -> Vec3 {
        self.dir
    }
}
