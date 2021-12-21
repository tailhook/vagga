use std::collections::{BTreeMap, BTreeSet};
use std::default::Default;
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use libmount::{BindMount, Overlay};

use crate::config::{Config, Container, Settings};
use crate::container::util::clean_dir;
use crate::builder::commands::composer::ComposerConfig;
use crate::builder::commands::gem::GemConfig;
use crate::builder::commands::pip::PipConfig;
use crate::builder::commands::npm::NpmConfig;
use crate::capsule;
use crate::file_util::Dir;
use crate::process_util::PROXY_ENV_VARS;

use super::packages;
use super::timer;

pub struct CachedDir {
    pub src: PathBuf,
    pub overlay_dir: Option<PathBuf>,
}

pub struct Context<'a> {
    pub config: &'a Config,
    pub container_name: String,
    pub container_config: &'a Container,
    pub ensure_dirs: BTreeSet<PathBuf>,
    pub empty_dirs: BTreeSet<PathBuf>,
    pub remove_paths: BTreeSet<PathBuf>,
    pub mounted: Vec<PathBuf>,
    pub cache_dirs: BTreeMap<PathBuf, CachedDir>,
    pub environ: BTreeMap<String, String>,

    /// String that identifies binary API version
    ///
    /// Currenty we only put OS and release here, but in future we may add
    /// more useful things. Making it too chaotic will make a lot of cache
    /// that is not usable.
    pub binary_ident: String,

    pub settings: Arc<Settings>,
    pub pip_settings: PipConfig,
    pub gem_settings: GemConfig,
    pub npm_settings: NpmConfig,
    pub npm_configured: bool,
    pub composer_settings: ComposerConfig,
    pub capsule: capsule::State,
    pub packages: BTreeSet<String>,
    pub build_deps: BTreeSet<String>,
    pub featured_packages: BTreeSet<packages::Package>,
    pub network_namespace: Option<File>,
    pub timelog: timer::TimeLog,
}

impl<'a> Context<'a> {
    pub fn new<'x>(cfg: &'x Config, name: String,
        container: &'x Container, settings: Settings)
        -> Context<'x>
    {
        let mut env: BTreeMap<String, String> = vec!(
            ("TERM".to_string(), "dumb".to_string()),
            ("HOME".to_string(), "/tmp".to_string()),
            ("PATH".to_string(),
             "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
             .to_string()),
            ).into_iter().collect();
        if settings.proxy_env_vars {
            for k in &PROXY_ENV_VARS {
                if let Ok(v) = env::var(k) {
                    env.insert(k.to_string(), v);
                }
            }
        }
        let settings = Arc::new(settings);
        return Context {
            config: cfg,
            container_name: name,
            container_config: container,
            ensure_dirs: vec!(
                PathBuf::from("dev"),
                PathBuf::from("proc"),
                PathBuf::from("run"),
                PathBuf::from("sys"),
                PathBuf::from("tmp"),
                PathBuf::from("work"),
                ).into_iter().collect(),
            empty_dirs: vec!(
                PathBuf::from("tmp"),
                PathBuf::from("var/tmp"),
                ).into_iter().collect(),
            remove_paths: BTreeSet::new(),
            mounted: Vec::new(),
            cache_dirs: BTreeMap::new(),
            environ: env,
            binary_ident: "amd64".to_string(),
            settings: settings.clone(),
            pip_settings: Default::default(),
            gem_settings: Default::default(),
            npm_settings: Default::default(),
            npm_configured: false,
            composer_settings: Default::default(),
            capsule: capsule::State::new(&settings),
            packages: BTreeSet::new(),
            build_deps: BTreeSet::new(),
            featured_packages: BTreeSet::new(),
            network_namespace: None,
            timelog: timer::TimeLog::start(
                    &Path::new("/vagga/container/timings.log"))
                .map_err(|e| format!("Can't write timelog: {}", e))
                .unwrap(),
        };
    }

    pub fn add_cache_dir(&mut self, path: &Path, name: &str)
        -> Result<(), String>
    {
        if self.cache_dirs.contains_key(path) {
            return Ok(());
        }

        let cache_dir = Path::new("/vagga/cache").join(&name);
        if !cache_dir.exists() {
            try_msg!(
                Dir::new(&cache_dir).create(),
                "Error creating cache dir: {err}"
            );
        }

        let tgt_path = Self::cache_target_path(path)?;
        try_msg!(
            Dir::new(&tgt_path).recursive(true).create(),
            "Error creating cache dir: {err}"
        );
        clean_dir(&tgt_path, false)?;

        try_msg!(
            BindMount::new(&cache_dir, &tgt_path).mount(),
            "mount cache dir: {err}"
        );
        debug!("Cache mounted: {:?} -> {:?}", &cache_dir, &tgt_path);

        self.mounted.push(tgt_path);
        self.cache_dirs.insert(
            path.to_path_buf(),
            CachedDir { src: cache_dir, overlay_dir: None }
        );
        Ok(())
    }

    pub fn add_cache_dir_overlay(&mut self, path: &Path, name: &str)
        -> Result<(), String>
    {
        if self.cache_dirs.contains_key(path) {
            return Ok(());
        }

        let overlay_dir = Path::new("/vagga/container").join(name);
        let work_dir = overlay_dir.join("work");
        let upper_dir = overlay_dir.join("upper");
        try_msg!(
            Dir::new(&work_dir).recursive(true).create(),
            "Error creating cache work dir: {err}"
        );
        try_msg!(
            Dir::new(&upper_dir).recursive(true).create(),
            "Error creating cache upper dir: {err}"
        );

        let cache_dir = Path::new("/vagga/cache").join(&name);
        if !cache_dir.exists() {
            try_msg!(
                Dir::new(&cache_dir).create(),
                "Error creating cache dir: {err}"
            );
        }

        let tgt_path = Self::cache_target_path(path)?;
        try_msg!(
            Dir::new(&tgt_path).recursive(true).create(),
            "Error creating cache dir: {err}"
        );
        clean_dir(&tgt_path, false)?;

        try_msg!(
            Overlay::writable(
                vec!(cache_dir.as_path()).into_iter(),
                &upper_dir,
                &work_dir,
                tgt_path.as_path(),
            )
            .mount(),
            "mount cache dir: {err}"
        );
        debug!("Cache mounted: {:?} -> {:?}", &cache_dir, &tgt_path);

        self.mounted.push(tgt_path);
        self.cache_dirs.insert(
            path.to_path_buf(),
            CachedDir { src: cache_dir, overlay_dir: Some(overlay_dir) }
        );
        Ok(())
    }

    fn cache_target_path(path: &Path) -> Result<PathBuf, String> {
        let rel_path = path.strip_prefix("/")
            .map_err(|_| format!("cache_dir must be absolute: {:?}", path))?;
        Ok(Path::new("/vagga/root").join(rel_path))
    }

    pub fn get_cached_dir(&self, path: &Path) -> Option<&CachedDir> {
        self.cache_dirs.get(path)
    }

    pub fn is_cached_dir_overlay(&self, path: &Path) -> bool {
        self.get_cached_dir(path)
            .and_then(|d| d.overlay_dir.as_ref())
            .is_some()
    }

    pub fn add_remove_path(&mut self, path: &Path)
        -> Result<(), String>
    {
        let rel_path = path.strip_prefix("/")
            .map_err(|_| format!("remove path must be absolute: {:?}", path))?;
        self.remove_paths.insert(rel_path.to_path_buf());
        Ok(())
    }

    pub fn add_empty_dir(&mut self, path: &Path)
        -> Result<(), String>
    {
        let rel_path = path.strip_prefix("/")
            .map_err(|_| format!("empty_dir must be absolute: {:?}", path))?;
        self.empty_dirs.insert(rel_path.to_path_buf());
        Ok(())
    }

    pub fn add_ensure_dir(&mut self, path: &Path)
        -> Result<(), String>
    {
        let rel_path = path.strip_prefix("/")
            .map_err(|_| format!("ensure_dir must be absolute: {:?}", path))?;
        self.ensure_dirs.insert(rel_path.to_path_buf());
        Ok(())
    }

}
