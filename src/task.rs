use crate::{ChunkID, ComposeableGenerator, Generator, mpsc_channel, worker::Runable};

#[derive(Debug, Clone)]
pub struct Context {
    pub world_generator: ComposeableGenerator,
    pub meshes: mpsc_channel::Sender<()>,
}

#[derive(Debug)]
pub enum Task {
    GenerateChunk { chunk: ChunkID },
}

impl Runable<Context> for Task {
    fn run(self, debug_log: &mut Vec<String>, context: &mut Context) {
        use Task::*;
        match self {
            GenerateChunk { chunk } => generate_chunk(context, chunk),
        }
    }
}

pub fn generate_chunk(context: &mut Context, chunk: ChunkID) {
    let data = context.world_generator.generate(chunk);
}
