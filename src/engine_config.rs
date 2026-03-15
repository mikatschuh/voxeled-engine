use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub full_detail_distance: f32,
    pub total_generation_distance: f32,
    pub max_chunks: usize,

    pub print_tps: bool,

    pub worker_count: usize,

    pub task_queue_cap: usize,

    pub mesh_queue_cap: usize,
    pub chunk_queue_cap: usize,
    pub collider_queue_cap: usize,
    pub solid_map_queue_cap: usize,
}

#[derive(Deserialize, Serialize)]
pub struct ConfigUpdate {
    pub full_detail_distance: f32,
    pub total_generation_distance: f32,
    pub max_chunks: usize,

    pub print_tps: bool,
}

impl Config {
    pub fn update(&mut self, update: ConfigUpdate) {
        self.full_detail_distance = update.full_detail_distance;
        self.total_generation_distance = update.total_generation_distance;
        self.max_chunks = update.max_chunks;
        self.print_tps = update.print_tps;
    }
}
