use std::f32::consts::{FRAC_PI_2, PI};

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{DeltaTime, physics::TCBody};

pub fn dir_from_angle(yaw: f32, pitch: f32) -> Vec3 {
    let (sin_pitch, cos_pitch) = pitch.sin_cos();
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize()
}

fn wrap_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(2.0 * PI) - PI
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CameraConfig {
    friction: f32,
    standart_speed: f32,
    max_speed: f32,
    acc_change_sensitivity: f32,
    sensitivity: f32,
}

#[derive(Debug, Clone)]
pub struct CamController {
    body: TCBody,
    pending_acc: Vec3,

    free_cam: bool,
    speed: f32, // camera speed

    yaw: f32, // source of truth
    pitch: f32,
    inverted: bool,

    dir: Vec3,
    up: Vec3,

    delta_time: DeltaTime,
    config: CameraConfig,
}

impl CamController {
    pub fn new(
        pos: Vec3,
        yaw: f32,
        pitch: f32,
        free_cam: bool,
        delta_time: DeltaTime,
        config: CameraConfig,
    ) -> Self {
        let mut controller = Self {
            body: TCBody::new(pos),
            pending_acc: Vec3::ZERO,

            free_cam,
            speed: config.standart_speed,

            yaw,
            pitch,
            inverted: false,
            dir: Vec3::ZERO,
            up: Vec3::Y,

            delta_time,
            config,
        };

        controller.normalize_angles();
        controller.update_basis();
        controller
    }

    /// Dreht die Kamera um einen Winkel multipliziert mit der Kamera Sensitivität.
    pub fn rotate_around_angle(&mut self, yaw: f32, pitch: f32) {
        let control_sign = self.control_sign();
        self.yaw += yaw * self.config.sensitivity * control_sign;
        self.pitch += pitch * self.config.sensitivity * control_sign;

        self.normalize_angles();
        self.update_basis();
    }

    /// Bewegt die Kamera in eine Richtung relativ zur Richtung in die die Kamera zeigt.
    pub fn add_input(&mut self, input_vector: Vec3) {
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let forward = self.dir;
        let right = (Vec3::new(-yaw_sin, 0.0, yaw_cos) * self.control_sign()).normalize();
        let impuls = forward * input_vector.x * self.speed
            + right * input_vector.z * self.speed
            + self.up * input_vector.y * self.speed;

        self.pending_acc = impuls;
    }

    pub fn add_acc(&mut self, acc: Vec3) {
        self.pending_acc = acc;
    }

    /// Takes a function which takes the current and the next position and returns the resolved position.
    pub fn advance_pos(&mut self, contrain: impl FnMut(Vec3, Vec3) -> Vec3) {
        self.body.step(self.delta_time(), self.config.friction);

        let dt = self.delta_time();

        self.body
            .constrain(|_, next_pos| next_pos + self.pending_acc * dt / 2. * dt);

        self.body.constrain(contrain);
    }

    pub fn update_speed(&mut self, change: f32) {
        self.speed *= (self.config.acc_change_sensitivity * change).exp();

        self.speed = self.speed.clamp(
            self.config.standart_speed / self.config.max_speed,
            self.config.standart_speed * self.config.max_speed,
        );

        println!("new speed: {}", self.speed);
    }

    pub fn update_config(&mut self, config: CameraConfig) {
        self.config = config
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

    pub fn up(&self) -> Vec3 {
        self.up
    }

    fn update_basis(&mut self) {
        self.dir = dir_from_angle(self.yaw, self.pitch);
        self.up = Vec3::Y * self.control_sign();
    }

    fn normalize_angles(&mut self) {
        while self.pitch > FRAC_PI_2 {
            self.pitch = PI - self.pitch;
            self.yaw += PI;
            self.inverted = !self.inverted;
        }

        while self.pitch < -FRAC_PI_2 {
            self.pitch = -PI - self.pitch;
            self.yaw += PI;
            self.inverted = !self.inverted;
        }

        self.yaw = wrap_angle(self.yaw);
    }

    fn control_sign(&self) -> f32 {
        if self.inverted { -1.0 } else { 1.0 }
    }
}
