use glam::{IVec3, Vec3};

use crate::physics::block;

pub trait Voxel {
    fn solid_at(&self, pos: IVec3) -> bool;

    fn check_volume_for_collision(&self, (start_corner, end_corner): (IVec3, IVec3)) -> bool {
        (start_corner.x..=end_corner.x)
            .flat_map(move |x| {
                (start_corner.y..=end_corner.y)
                    .flat_map(move |y| (start_corner.z..=end_corner.z).map(move |z| (x, y, z)))
            })
            .any(|(x, y, z)| self.solid_at(IVec3::new(x, y, z)))
    }
}

const PLAYER_HALF_EXTENTS: Vec3 = Vec3::new(0.3, 0.9, 0.3);

const EPSILON: f32 = 0.00001;

#[derive(Clone, Debug, PartialEq)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    pub fn player(pos: Vec3) -> Self {
        Self {
            min: pos - PLAYER_HALF_EXTENTS,
            max: pos + PLAYER_HALF_EXTENTS,
        }
    }

    pub fn new(pos: Vec3, half_extends: Vec3) -> Self {
        Self {
            min: pos - half_extends,
            max: pos + half_extends,
        }
    }

    pub fn player_pos(&self) -> Vec3 {
        self.min + PLAYER_HALF_EXTENTS
    }

    pub fn step(&mut self, delta: Vec3) {
        self.min += delta;
        self.max += delta;
    }

    pub fn step_x(&mut self, delta: f32) {
        self.min.x += delta;
        self.max.x += delta;
    }

    pub fn step_y(&mut self, delta: f32) {
        self.min.y += delta;
        self.max.y += delta;
    }

    pub fn step_z(&mut self, delta: f32) {
        self.min.z += delta;
        self.max.z += delta;
    }

    fn corners_blocked(&self) -> (IVec3, IVec3) {
        (block(self.min), block(self.max))
    }

    fn corners(&self) -> (Vec3, Vec3) {
        (self.min, self.max)
    }

    pub fn sweep_through_voxel(
        &mut self,
        voxel: &impl Voxel,
        mut delta: Vec3,
        mut material_coef: f32,
    ) -> Vec3 {
        if voxel.check_volume_for_collision(self.corners_blocked()) {
            return self.player_pos() + delta;
        }

        loop {
            let max_element = delta.abs().max_element();
            let step = if max_element > 1. {
                delta / max_element
            } else if max_element < EPSILON {
                return self.player_pos();
            } else {
                delta
            };

            let x_positive = step.x.is_sign_positive();
            let y_positive = step.y.is_sign_positive();
            let z_positive = step.z.is_sign_positive();

            // create check on x axis
            let x = if x_positive { self.max } else { self.min }.x;
            let x_space = 1. - x.abs().fract() - EPSILON;
            let x_check = if x_space < step.x.abs() {
                let check_x = x + step.x.signum();
                Some((
                    block(Vec3::new(check_x, self.min.y, self.min.z)),
                    block(Vec3::new(check_x, self.max.y, self.max.z)),
                ))
            } else {
                None
            };

            // create check on y axis
            let y = if y_positive { self.max } else { self.min }.y;
            let y_space = 1. - y.abs().fract() - EPSILON;
            let y_check = if y_space < step.y.abs() {
                let check_y = y + step.y.signum();
                Some((
                    block(Vec3::new(self.min.x, check_y, self.min.z)),
                    block(Vec3::new(self.max.x, check_y, self.max.z)),
                ))
            } else {
                None
            };

            // create check on z axis
            let z = if z_positive { self.max } else { self.min }.z;
            let z_space = 1. - z.abs().fract() - EPSILON;
            let z_check = if z_space < step.z.abs() {
                let check_z = z + step.z.signum();
                Some((
                    block(Vec3::new(self.min.x, self.min.y, check_z)),
                    block(Vec3::new(self.max.x, self.max.y, check_z)),
                ))
            } else {
                None
            };

            if x_check.is_some_and(|check| voxel.check_volume_for_collision(check)) {
                let remainder = step.x.signum() * x_space;

                delta.x -= remainder;
                self.step_x(remainder);
                delta.x *= -material_coef;
            } else {
                delta.x -= step.x;
                self.step_x(step.x);
            }

            if y_check.is_some_and(|check| voxel.check_volume_for_collision(check)) {
                let remainder = step.y.signum() * y_space;

                delta.y -= remainder;
                self.step_y(remainder);
                delta.y *= -material_coef;
            } else {
                delta.y -= step.y;
                self.step_y(step.y);
            }

            if z_check.is_some_and(|check| voxel.check_volume_for_collision(check)) {
                let remainder = step.z.signum() * z_space;

                delta.z -= remainder;
                self.step_z(remainder);
                delta.z *= -material_coef;
            } else {
                delta.z -= step.z;
                self.step_z(step.z);
            }
        }
    }

    pub fn sweep_through_voxel_and_collide_per_axis(
        &mut self,
        voxel: &impl Voxel,
        mut delta: Vec3,
        mut material_coef: f32,
    ) -> Vec3 {
        loop {
            let max_element = delta.abs().max_element();
            let step = if max_element > 1. {
                delta / max_element
            } else if max_element < EPSILON {
                return self.player_pos();
            } else {
                delta
            };

            let x_positive = step.x.is_sign_positive();
            let y_positive = step.y.is_sign_positive();
            let z_positive = step.z.is_sign_positive();

            // create check on x axis
            let x = if x_positive { self.max } else { self.min }.x;
            let x_space = 1. - x.abs().fract() - EPSILON;
            let x_check = if x_space < step.x.abs() {
                let check_x = x + step.x.signum();
                Some((
                    block(Vec3::new(check_x, self.min.y, self.min.z)),
                    block(Vec3::new(check_x, self.max.y, self.max.z)),
                ))
            } else {
                None
            };

            // process movement
            if x_check.is_some_and(|check| voxel.check_volume_for_collision(check)) {
                let remainder = step.x.signum() * x_space;

                delta.x -= remainder;
                self.step_x(remainder);
                delta.x *= -material_coef;
            } else {
                delta.x -= step.x;
                self.step_x(step.x);
            }

            // create check on y axis
            let y = if y_positive { self.max } else { self.min }.y;
            let y_space = 1. - y.abs().fract() - EPSILON;
            let y_check = if y_space < step.y.abs() {
                let check_y = y + step.y.signum();
                Some((
                    block(Vec3::new(self.min.x, check_y, self.min.z)),
                    block(Vec3::new(self.max.x, check_y, self.max.z)),
                ))
            } else {
                None
            };

            // process movement
            if y_check.is_some_and(|check| voxel.check_volume_for_collision(check)) {
                let remainder = step.y.signum() * y_space;

                delta.y -= remainder;
                self.step_y(remainder);
                delta.y *= -material_coef;
            } else {
                delta.y -= step.y;
                self.step_y(step.y);
            }

            // create check on z axis
            let z = if z_positive { self.max } else { self.min }.z;
            let z_space = 1. - z.abs().fract() - EPSILON;
            let z_check = if z_space < step.z.abs() {
                let check_z = z + step.z.signum();
                Some((
                    block(Vec3::new(self.min.x, self.min.y, check_z)),
                    block(Vec3::new(self.max.x, self.max.y, check_z)),
                ))
            } else {
                None
            };

            // process movement
            if z_check.is_some_and(|check| voxel.check_volume_for_collision(check)) {
                let remainder = step.z.signum() * z_space;

                delta.z -= remainder;
                self.step_z(remainder);
                delta.z *= -material_coef;
            } else {
                delta.z -= step.z;
                self.step_z(step.z);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use glam::{IVec3, Vec3};

    use crate::physics::Aabb;
    use crate::physics::Voxel;

    struct SingleSolid(IVec3);

    impl Voxel for SingleSolid {
        fn solid_at(&self, pos: IVec3) -> bool {
            pos == self.0
        }
    }
}
