use glam::Vec3;

pub struct TCBody {
    prev_pos: Vec3,
    prev_time: f32,

    pos: Vec3,
}

impl TCBody {
    pub fn new(pos: Vec3) -> Self {
        Self {
            prev_pos: pos,
            prev_time: 0.1666,
            pos,
        }
    }

    pub fn step(&mut self, time: f32, damping_coef: f32) {
        let ds = (self.pos - self.prev_pos) / self.prev_time * time; // v = ds/dt; v * dt = ds

        self.prev_pos = self.pos;
        self.prev_time = time;
        self.pos += ds * (-damping_coef * time).exp();
    }

    pub fn constrain(&mut self, mut constrain: impl FnMut(Vec3, Vec3) -> Vec3) {
        self.pos = constrain(self.prev_pos, self.pos)
    }

    pub fn pos(&self) -> Vec3 {
        self.pos
    }
}

pub struct Body {
    prev_pos: Vec3,
    pos: Vec3,

    pending_impuls: Vec3,
}

impl Body {
    pub fn new(pos: Vec3) -> Self {
        Self {
            prev_pos: pos,
            pos,
            pending_impuls: Vec3::ZERO,
        }
    }

    pub fn add_impuls(&mut self, acc: Vec3) {
        self.pending_impuls += acc
    }

    pub fn step_time(&mut self, damping_coef: f32) {
        let vel = self.pos - self.prev_pos;

        self.prev_pos = self.pos;
        self.pos += (vel + self.pending_impuls) * (-damping_coef).exp();
        self.pending_impuls = Vec3::ZERO;
    }

    pub fn constrain(&mut self, mut constrain: impl FnMut(Vec3, Vec3) -> Vec3) {
        self.pos = constrain(self.prev_pos, self.pos)
    }

    pub fn pos(&self) -> Vec3 {
        self.pos
    }
}
