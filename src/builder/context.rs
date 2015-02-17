use std::io::ALL_PERMISSIONS;
use std::io::fs::{mkdir_recursive, mkdir, copy};
use std::io::fs::PathExtensions;
use std::default::Default;
use std::collections::{BTreeMap, BTreeSet};

use container::mount::{bind_mount, unmount, mount_system_dirs};
use container::util::clean_dir;
use config::Config;
use config::Container;
use config::Settings;
use config::builders::PipSettings;
use super::commands::debian::UbuntuInfo;
use super::commands::alpine::AlpineInfo;
use super::commands::debian;
use super::commands::alpine;

#[derive(Show)]
pub enum Distribution {
    Unknown,
    Ubuntu(UbuntuInfo),
    Alpine(AlpineInfo),
}

pub struct BuildContext<'a> {
    pub config: &'a Config,
    pub container_name: String,
    pub container_config: &'a Container,
    ensure_dirs: BTreeSet<Path>,
    empty_dirs: BTreeSet<Path>,
    remove_dirs: BTreeSet<Path>,
    cache_dirs: BTreeMap<Path, String>,
    pub environ: BTreeMap<String, String>,

    pub settings: Settings,
    pub distribution: Distribution,
    pub pip_settings: PipSettings,
    pub packages: BTreeSet<String>,
    pub build_deps: BTreeSet<String>,
}

impl<'a> BuildContext<'a> {
    pub fn new<'x>(cfg: &'x Config, name: String,
        container: &'x Container, settings: Settings)
        -> BuildContext<'x>
    {
        return BuildContext {
            config: cfg,
            container_name: name,
            container_config: container,
            ensure_dirs: vec!(
                Path::new("proc"),
                Path::new("sys"),
                Path::new("dev"),
                Path::new("work"),
                Path::new("tmp"),
                ).into_iter().collect(),
            empty_dirs: vec!(
                Path::new("tmp"),
                Path::new("var/tmp"),
                ).into_iter().collect(),
            remove_dirs: vec!(
                ).into_iter().collect(),
            cache_dirs: vec!(
                ).into_iter().collect(),
            environ: vec!(
                ("TERM".to_string(), "dumb".to_string()),
                ("HOME".to_string(), "/tmp".to_string()),
                ("PATH".to_string(),
                 "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
                 .to_string()),
                ).into_iter().collect(),

            settings: settings,
            distribution: Distribution::Unknown,
            pip_settings: Default::default(),
            packages: BTreeSet::new(),
            build_deps: BTreeSet::new(),
        };
    }

    pub fn add_cache_dir(&mut self, path: Path, name: String)
        -> Result<(), String>
    {
        assert!(path.is_absolute());
        let path = path.path_relative_from(&Path::new("/")).unwrap();
        if self.cache_dirs.insert(path.clone(), name.clone()).is_none() {
            let cache_dir = Path::new("/vagga/cache").join(name.as_slice());
            if !cache_dir.exists() {
                try!(mkdir(&cache_dir, ALL_PERMISSIONS)
                     .map_err(|e| format!("Error creating cache dir: {}", e)));
            }
            let path = Path::new("/vagga/root").join(path);
            try!(mkdir_recursive(&path, ALL_PERMISSIONS)
                 .map_err(|e| format!("Error creating cache dir: {}", e)));
            try!(clean_dir(&path, false));
            try!(bind_mount(&cache_dir, &path));
        }
        return Ok(());
    }

    pub fn add_remove_dir(&mut self, path: Path) {
        assert!(path.is_absolute());
        let path = path.path_relative_from(&Path::new("/")).unwrap();
        self.remove_dirs.insert(path);
    }

    pub fn add_empty_dir(&mut self, path: Path) {
        assert!(path.is_absolute());
        let path = path.path_relative_from(&Path::new("/")).unwrap();
        self.empty_dirs.insert(path);
    }

    pub fn add_ensure_dir(&mut self, path: Path) {
        assert!(path.is_absolute());
        let path = path.path_relative_from(&Path::new("/")).unwrap();
        self.ensure_dirs.insert(path);
    }
    pub fn start(&mut self) -> Result<(), String> {
        try!(mount_system_dirs());
        try!(mkdir(&Path::new("/vagga/root/etc"), ALL_PERMISSIONS)
             .map_err(|e| format!("Error creating /etc dir: {}", e)));
        try!(copy(&Path::new("/etc/resolv.conf"),
                  &Path::new("/vagga/root/etc/resolv.conf"))
            .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), String> {

        match self.distribution {
            Distribution::Unknown => {}
            Distribution::Ubuntu(_) => {
                try!(debian::finish(self));
            }
            Distribution::Alpine(_) => {
                try!(alpine::finish(self));
            }
        }

        let base = Path::new("/vagga/root");

        for (dir, _) in self.cache_dirs.iter().rev() {
            try!(unmount(&base.join(dir)));
        }

        for dir in self.remove_dirs.iter() {
            try!(clean_dir(&base.join(dir), false)
                .map_err(|e| format!("Error removing dir: {}", e)));
        }

        for dir in self.empty_dirs.iter() {
            try!(clean_dir(&base.join(dir), false));
        }

        for dir in self.ensure_dirs.iter() {
            try!(mkdir_recursive(&base.join(dir), ALL_PERMISSIONS)
                .map_err(|e| format!("Error creating dir: {}", e)));
        }

        return Ok(());
    }
}
