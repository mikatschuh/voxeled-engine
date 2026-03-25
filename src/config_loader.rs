use std::{
    fmt::Debug,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Serialize, de::DeserializeOwned};

use crate::error::{ConfigError, ConfigResult};

pub trait Live: Clone + Send + Sync + 'static {}
pub trait ConfigFile<L: Live, C: Config<L>, E: Into<ConfigError>>:
    DeserializeOwned + Serialize + Clone + Send + Sync + 'static
{
    fn check(self) -> Result<C, E>;
}
pub trait Config<L> {
    fn live(self) -> L;
    fn sender_cap(&self) -> usize;
}

pub fn config_thread<CF: ConfigFile<L, C, E> + Debug, C: Config<L>, L: Live, E>(
    path: PathBuf,
) -> ConfigResult<(C, rtrb::Consumer<L>)>
where
    ConfigError: From<E>,
{
    let mut settings_file = File::open(&path)?;
    let mut toml_settings = String::new();
    settings_file.read_to_string(&mut toml_settings)?;

    let initial_config = toml::from_str::<CF>(&toml_settings)?.check()?;

    let (mut main_tx, main_rx) = rtrb::RingBuffer::<L>::new(initial_config.sender_cap());

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    watcher.watch(&path, RecursiveMode::NonRecursive)?;

    let mut last_hash = None;
    let mut last_event = Instant::now();

    thread::Builder::new()
        .name("config thread".to_owned())
        .spawn(move || -> () {
            loop {
                rx.recv().unwrap().unwrap();

                // debounce
                if last_event.elapsed() < Duration::from_millis(100) {
                    continue;
                }
                last_event = Instant::now();

                let data = match fs::read(&path) {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                let hash = blake3::hash(&data);

                if Some(hash) == last_hash {
                    continue;
                }

                last_hash = Some(hash);

                let Ok(string) = str::from_utf8(&data) else {
                    continue;
                };

                match toml::from_str::<CF>(string) {
                    Ok(cfg) => {
                        let Ok(cfg) = cfg.check() else { continue };
                        _ = main_tx.push(cfg.live());
                    }
                    Err(_) => {}
                }

                if false {
                    break;
                }
            }
            drop(watcher);
        })?;

    Ok((initial_config, main_rx))
}
