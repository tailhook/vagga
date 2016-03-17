use std::env;
use std::path::{Path, PathBuf};
use std::default::Default;
use std::collections::{BTreeMap, BTreeSet};

use container::mount::{bind_mount};
use container::util::clean_dir;
use config::Config;
use config::Container;
use config::Settings;
use config::builders::{
    ComposerSettings,
    GemSettings,
    PipSettings,
    NpmSettings
};
use super::capsule;
use super::packages;
use super::timer;
use path_util::ToRelative;
use file_util::create_dir;
use process_util::PROXY_ENV_VARS;


pub struct Context<'a> {
    pub config: &'a Config,
    pub container_name: String,
    pub container_config: &'a Container,
    pub ensure_dirs: BTreeSet<PathBuf>,
    pub empty_dirs: BTreeSet<PathBuf>,
    pub remove_dirs: BTreeSet<PathBuf>,
    pub mounted: Vec<PathBuf>,
    pub cache_dirs: BTreeMap<PathBuf, String>,
    pub environ: BTreeMap<String, String>,

    /// String that identifies binary API version
    ///
    /// Currenty we only put OS and release here, but in future we may add
    /// more useful things. Making it too chaotic will make a lot of cache
    /// that is not usable.
    pub binary_ident: String,

    pub settings: Settings,
    pub pip_settings: PipSettings,
    pub gem_settings: GemSettings,
    pub npm_settings: NpmSettings,
    pub composer_settings: ComposerSettings,
    pub capsule: capsule::State,
    pub packages: BTreeSet<String>,
    pub build_deps: BTreeSet<String>,
    pub featured_packages: BTreeSet<packages::Package>,
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
        return Context {
            config: cfg,
            container_name: name,
            container_config: container,
            ensure_dirs: vec!(
                PathBuf::from("proc"),
                PathBuf::from("sys"),
                PathBuf::from("dev"),
                PathBuf::from("work"),
                PathBuf::from("tmp"),
                PathBuf::from("run"),
                ).into_iter().collect(),
            empty_dirs: vec!(
                PathBuf::from("tmp"),
                PathBuf::from("var/tmp"),
                ).into_iter().collect(),
            remove_dirs: BTreeSet::new(),
            mounted: Vec::new(),
            cache_dirs: BTreeMap::new(),
            environ: env,
            binary_ident: "amd64".to_string(),
            settings: settings,
            pip_settings: Default::default(),
            gem_settings: Default::default(),
            npm_settings: Default::default(),
            composer_settings: Default::default(),
            capsule: Default::default(),
            packages: BTreeSet::new(),
            build_deps: BTreeSet::new(),
            featured_packages: BTreeSet::new(),
            timelog: timer::TimeLog::start(
                    &Path::new("/vagga/container/timings.log"))
                .map_err(|e| format!("Can't write timelog: {}", e))
                .unwrap(),
        };
    }

    pub fn add_cache_dir(&mut self, path: &Path, name: String)
        -> Result<(), String>
    {
        assert!(path.is_absolute());
        let path = path.rel();
        if self.cache_dirs.insert(path.to_path_buf(), name.clone()).is_none() {
            let cache_dir = Path::new("/vagga/cache").join(&name);
            if !cache_dir.exists() {
                try_msg!(create_dir(&cache_dir, false),
                     "Error creating cache dir: {err}");
            }
            let path = Path::new("/vagga/root").join(path);
            try_msg!(create_dir(&path, true),
                 "Error creating cache dir: {err}");
            try!(clean_dir(&path, false));
            try!(bind_mount(&cache_dir, &path));
            self.mounted.push(path);
        }
        return Ok(());
    }

    pub fn add_remove_dir(&mut self, path: &Path) {
        assert!(path.is_absolute());
        self.remove_dirs.insert(path.rel().to_path_buf());
    }

    pub fn add_empty_dir(&mut self, path: &Path) {
        assert!(path.is_absolute());
        self.empty_dirs.insert(path.rel().to_path_buf());
    }

    pub fn add_ensure_dir(&mut self, path: &Path) {
        assert!(path.is_absolute());
        self.ensure_dirs.insert(path.rel().to_path_buf());
    }

}
