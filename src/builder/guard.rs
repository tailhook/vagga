use std::path::Path;
use std::io::BufWriter;
use std::fs::{File, remove_file};
use std::collections::HashMap;

use dir_signature::{v1, ScannerConfig as Sig};
use dir_signature::HashType;

use builder::context::Context;
use builder::distrib::{Unknown,Distribution};
use builder::error::{Error};
use builder::commands::{composer, gem, npm, pip, dirs};
use builder::packages;
use build_step::BuildStep;
use container::util::clean_dir;
use container::mount::{unmount, mount_system_dirs, mount_proc, mount_run};
use container::mount::unmount_system_dirs;
use file_util::{Dir, copy, truncate_file};
use path_util::IterSelfAndParents;
use capsule;


pub struct Guard<'a> {
    pub ctx: Context<'a>,
    pub distro: Box<Distribution>,
}


impl<'a> Guard<'a> {
    pub fn build(ctx: Context) -> Result<(), Error> {
        Guard {
            ctx: ctx,
            distro: Box::new(Unknown),
        }.run_process()
    }

    fn run_process(&mut self) -> Result<(), Error> {
        self.start()?;

        let mut downloads = Vec::new();
        for b in self.ctx.container_config.setup.iter() {
            b.get_downloads(&mut downloads);
        }
        if downloads.len() > 0 {
            // TODO(tailhook) check existing files
            capsule::fetch::fetch_many(downloads);
            self.ctx.timelog.mark(format_args!("Download finished"))
                .map_err(|e| format!("Can't write timelog: {}", e))?;
        }


        for b in self.ctx.container_config.setup.iter() {
            debug!("Building step: {:?}", b);
            self.ctx.timelog.mark(format_args!("Step: {:?}", b))
                .map_err(|e| format!("Can't write timelog: {}", e))?;
            b.build(self, true)
                .map_err(|e| Error::Step(b.0.clone(), e))?;
        }

        self.finish()?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), String> {
        mount_system_dirs()?;
        mount_run(&Path::new("/vagga/root/run"))?;
        mount_proc(&Path::new("/proc"))?;
        copy("/proc/self/uid_map", "/vagga/container/uid_map")
            .map_err(|e| format!("Error copying uid_map: {}", e))?;
        copy("/proc/self/gid_map", "/vagga/container/gid_map")
            .map_err(|e| format!("Error copying gid_map: {}", e))?;
        try_msg!(Dir::new("/vagga/root/etc").create(),
             "Error creating /etc dir: {err}");
        copy("/etc/resolv.conf", "/vagga/root/etc/resolv.conf")
            .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e))?;
        copy("/etc/hosts", "/vagga/root/etc/hosts")
            .map_err(|e| format!("Error copying /etc/hosts: {}", e))?;
        self.ctx.timelog.mark(format_args!("Prepare"))
            .map_err(|e| format!("Can't write timelog: {}", e))?;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), String> {
        // Pip
        if self.ctx.featured_packages.contains(&packages::PipPy2) ||
           self.ctx.featured_packages.contains(&packages::PipPy3)
        {
            pip::freeze(&mut self.ctx)?;
        }
        // Npm
        if self.ctx.featured_packages.contains(&packages::Npm) {
            npm::list(&mut self.ctx)?;
        }
        // Composer
        if self.ctx.featured_packages.contains(&packages::Composer) {
            composer::finish(&mut self.ctx)?;
        }
        // Gem
        if self.ctx.featured_packages.contains(&packages::Bundler) {
            gem::list(&mut self.ctx)?;
        }

        self.distro.finish(&mut self.ctx)?;

        let base = Path::new("/vagga/root");

        for path in self.ctx.mounted.iter().rev() {
            unmount(path)?;
        }
        unmount(&Path::new("/vagga/root/run"))?;
        unmount_system_dirs()?;

        // Truncate resolv.conf and hosts files
        truncate_file("/vagga/root/etc/resolv.conf")?;
        if let Some(ref resolv_path) =
            self.ctx.container_config.resolv_conf_path
        {
            if resolv_path != Path::new("/etc/resolv.conf") {
                truncate_file(
                    &base.join(resolv_path.strip_prefix("/").unwrap()))?;
            }
        }
        truncate_file("/vagga/root/etc/hosts")?;
        if let Some(ref hosts_path) =
            self.ctx.container_config.hosts_file_path
        {
            if hosts_path != Path::new("/etc/hosts") {
                truncate_file(
                    &base.join(hosts_path.strip_prefix("/").unwrap()))?;
            }
        }

        for path in self.ctx.remove_paths.iter() {
            try_msg!(dirs::remove(&Path::new("/").join(path)),
                "Error removing path: {err}");
        }

        for dir in self.ctx.empty_dirs.iter() {
            clean_dir(&base.join(dir), false)?;
        }

        for dir in self.ctx.ensure_dirs.iter() {
            try_msg!(dirs::ensure(&Path::new("/").join(dir)),
                "Error creating dir: {err}");
        }

        if self.ctx.container_config.is_data_container() {
            let root = Path::new("/vagga/root");
            let exclude_paths = self.ctx.container_config.data_dirs.iter()
                // We validate exclude paths as absolute
                .map(|p| p.strip_prefix("/").unwrap())
                .collect::<Vec<_>>();
            let mut keep_rel_paths = HashMap::new();
            for exclude_path in &exclude_paths {
                for p in exclude_path
                    .iter_self_and_parents().skip(1)
                {
                    if let Some(&true) = keep_rel_paths.get(p) {
                        warn_duplicate_data_dir(p, true);
                    }
                    if !keep_rel_paths.contains_key(p) {
                        keep_rel_paths.insert(p, false);
                    }
                }
                if let Some(&is_final) = keep_rel_paths.get(exclude_path) {
                    warn_duplicate_data_dir(exclude_path, is_final);
                }
                // true means final path
                // so we merely keep this directory
                // and do not process its subdirs
                keep_rel_paths.insert(exclude_path, true);
            }
            try_msg!(remove_all_except(root, &keep_rel_paths),
                "Error removing dirs: {err}");
        }

        if self.ctx.settings.index_all_images {
            self.ctx.timelog.mark(format_args!("Indexing"))
                .map_err(|e| format!("Can't write timelog: {}", e))?;
            index_image()?;
        }

        self.ctx.timelog.mark(format_args!("Finish"))
            .map_err(|e| format!("Can't write timelog: {}", e))?;

        return Ok(());
    }
}

fn remove_all_except(root: &Path, keep_rel_paths: &HashMap<&Path, bool>)
    -> Result<(), String>
{
    let entries = try_msg!(root.read_dir(),
        "Can't read dir {dir:?}: {err}", dir=root);
    for entry in entries {
        let ref path = try_msg!(entry,
                "Can't iterate over dir entries {dir:?}: {err}",
                dir=root)
            .path();
        let ref rel_path = path.strip_prefix("/vagga/root").unwrap();
        match keep_rel_paths.get(rel_path) {
            Some(&is_final) if is_final => {
                continue;
            },
            Some(_) => {
                remove_all_except(path, keep_rel_paths)?;
            },
            None => {
                if path.is_dir() {
                    try_msg!(clean_dir(path, true),
                        "Error cleaning dir {path:?}: {err}",
                        path=path);
                } else {
                    try_msg!(remove_file(path),
                        "Error removing file {path:?}: {err}",
                        path=path);
                }
            },
        }
    }
    Ok(())
}

fn warn_duplicate_data_dir(rel_path: &Path, is_final: bool) {
    let path = Path::new("/").join(rel_path);
    if is_final {
        warn!("{:?} is already contained as data directory", path);
    } else {
        warn!("{:?} is a prefix of other directory", path);
    }
}

pub fn index_image() -> Result<(), String> {
    let index = File::create("/vagga/container/index.ds1")
        .map_err(|e| format!("Can't write index: {}", e))?;
    warn!("Indexing container...");
    v1::scan(Sig::new()
            .hash(HashType::blake2b_256())
            .add_dir("/vagga/root", "/"),
        &mut BufWriter::new(index)
    ).map_err(|e| format!("Error indexing: {}", e))
}
