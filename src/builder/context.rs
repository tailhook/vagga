use std::fs::{create_dir_all, copy, set_permissions};
use std::fs::Permissions;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
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
use super::commands::pip;
use super::commands::npm;
use super::capsule;
use super::packages;
use super::timer;
use path_util::{PathExt, ToRelative};
use file_util::create_dir;

#[derive(Debug)]
pub enum Distribution {
    Unknown,
    Ubuntu(UbuntuInfo),
    Alpine(AlpineInfo),
}

pub struct BuildContext<'a> {
    pub config: &'a Config,
    pub container_name: String,
    pub container_config: &'a Container,
    ensure_dirs: BTreeSet<PathBuf>,
    empty_dirs: BTreeSet<PathBuf>,
    remove_dirs: BTreeSet<PathBuf>,
    cache_dirs: BTreeMap<PathBuf, String>,
    pub environ: BTreeMap<String, String>,

    pub settings: Settings,
    pub distribution: Distribution,
    pub pip_settings: PipSettings,
    pub capsule: capsule::State,
    pub packages: BTreeSet<String>,
    pub build_deps: BTreeSet<String>,
    pub featured_packages: BTreeSet<packages::Package>,
    pub timelog: timer::TimeLog,
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
                PathBuf::from("proc"),
                PathBuf::from("sys"),
                PathBuf::from("dev"),
                PathBuf::from("work"),
                PathBuf::from("tmp"),
                ).into_iter().collect(),
            empty_dirs: vec!(
                PathBuf::from("tmp"),
                PathBuf::from("var/tmp"),
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
    pub fn start(&mut self) -> Result<(), String> {
        try!(mount_system_dirs());
        try_msg!(create_dir(&Path::new("/vagga/root/etc"), false),
             "Error creating /etc dir: {err}");
        try!(copy("/etc/resolv.conf", "/vagga/root/etc/resolv.conf")
            .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));
        try!(self.timelog.mark(format_args!("Prepare"))
            .map_err(|e| format!("Can't write timelog: {}", e)));
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), String> {
        if self.featured_packages.contains(&packages::PipPy2) ||
           self.featured_packages.contains(&packages::PipPy3)
        {
            try!(pip::freeze(self));
        }
        if self.featured_packages.contains(&packages::Npm) {
            try!(npm::list(self));
        }

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
            let fulldir = base.join(dir);
            try_msg!(create_dir(&fulldir, true),
                "Error creating dir: {err}");
        }

        try!(self.timelog.mark(format_args!("Finish"))
            .map_err(|e| format!("Can't write timelog: {}", e)));

        return Ok(());
    }
}
