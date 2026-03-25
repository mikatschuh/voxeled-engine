use serde::{Deserialize, Serialize};

use crate::Lod;

/// This is the configuration for the engine thread
#[derive(Deserialize, Serialize)]
pub struct EngineConfig {
    pub full_detail_distance: f32,
    pub task_cancelation_lod_threshold: Lod,

    pub total_generation_distance: f32,
    pub max_chunks: usize,

    pub print_tps_per: Option<f64>,
    pub target_tps: f64,

    pub worker_count: usize,

    pub engine_worker_config_queue_cap: usize,
    pub task_queue_cap: usize,
    pub discarded_tasks_queue_cap: usize,
    pub mesh_queue_cap: usize,
    pub chunk_queue_cap: usize,
    pub collider_queue_cap: usize,
    pub solid_map_queue_cap: usize,
}

/// This are the parts of the configuration of the engine thread that can be changed live
#[derive(Deserialize, Serialize)]
pub struct ConfigUpdate {
    pub full_detail_distance: f32,
    pub task_cancelation_lod_threshold: Lod,

    pub total_generation_distance: f32,
    pub max_chunks: usize,

    pub print_tps_per: Option<f64>,
    pub target_tps: f64,
}

impl ConfigUpdate {
    pub fn worker_config(&self) -> WorkerConfig {
        WorkerConfig {
            task_cancelation_lod_threshold: self.task_cancelation_lod_threshold,
            full_detail_distance: self.full_detail_distance,
        }
    }
}

impl EngineConfig {
    pub fn update(&mut self, update: ConfigUpdate) {
        let ConfigUpdate {
            full_detail_distance,
            task_cancelation_lod_threshold,
            total_generation_distance,
            max_chunks,
            print_tps_per,
            target_tps,
        } = update;

        self.full_detail_distance = full_detail_distance;
        self.task_cancelation_lod_threshold = task_cancelation_lod_threshold;

        self.total_generation_distance = total_generation_distance;
        self.max_chunks = max_chunks;
        self.print_tps_per = print_tps_per;
        self.target_tps = target_tps;
    }

    pub fn worker_config(&self) -> WorkerConfig {
        WorkerConfig {
            task_cancelation_lod_threshold: self.task_cancelation_lod_threshold,
            full_detail_distance: self.full_detail_distance,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub task_cancelation_lod_threshold: u16,
    pub full_detail_distance: f32,
}
