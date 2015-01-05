use std::io::ALL_PERMISSIONS;
use std::os::{getenv};
use std::io::BufferedReader;
use std::os::{self_exe_path};
use std::io::{TypeSymlink, TypeDirectory, FileNotFound};
use std::io::fs::{File, PathExtensions};
use std::io::fs::{mkdir, copy, readlink, symlink};
use std::collections::TreeMap;

use config::Container;
use config::containers::{Tmpfs};
use container::root::{change_root};
use container::mount::{bind_mount, unmount, mount_system_dirs, remount_ro};
use container::mount::{mount_tmpfs, mount_pseudo};
use super::run::DEFAULT_PATH;
use settings::{MergedSettings};


fn create_storage_dir(storage_dir: &Path, project_root: &Path)
    -> Result<Path, String>
{
    let name = match project_root.filename_str() {
        Some(name) => name,
        None => return Err(format!(
            "Project dir `{}` is either root or has bad characters",
            project_root.display())),
    };
    let path = storage_dir.join(name);
    if !path.exists() {
        return Ok(path);
    }
    for i in range(1i, 101i) {
        let result = format!("{}-{}", name, i);
        let path = storage_dir.join(result);
        if !path.exists() {
            return Ok(path);
        }
    }
    return Err(format!("Too many similar paths named {} in {}",
        name, storage_dir.display()));
}


fn make_cache_dir(_project_root: &Path, vagga_base: &Path,
    settings: &MergedSettings)
    -> Result<Path, String>
{
    match settings.cache_dir {
        Some(ref dir) if settings.shared_cache => {
            if !dir.exists() {
                return Err(format!(concat!("Cache directory `{}` must exists.",
                    " Please either create it or remove that configuration",
                    " setting"), dir.display()));
            }
            return Ok(dir.clone());
        }
        _ => {
            let dir = vagga_base.join(".cache");
            try!(safe_ensure_dir(&dir));
            return Ok(dir);
        }
   }
}

fn safe_ensure_dir(dir: &Path) -> Result<(), String> {
    match dir.lstat() {
        Ok(stat) if stat.kind == TypeSymlink => {
            return Err(format!(concat!("The `{0}` dir can't be a symlink. ",
                               "Please run `unlink {0}`"), dir.display()));
        }
        Ok(stat) if stat.kind == TypeDirectory => {
            // ok
        }
        Ok(_) => {
            return Err(format!(concat!("The `{0}` must be a directory. ",
                               "Please run `unlink {0}`"), dir.display()));
        }
        Err(ref e) if e.kind == FileNotFound => {
            try!(mkdir(dir, ALL_PERMISSIONS)
                .map_err(|e| format!("Can't create `{}`: {}",
                                     dir.display(), e)));
        }
        Err(ref e) => {
            return Err(format!("Can't stat `{}`: {}", dir.display(), e));
        }
    }
    return Ok(());
}

fn vagga_base(project_root: &Path, settings: &MergedSettings)
    -> Result<Path, String>
{
    if let Some(ref dir) = settings.storage_dir {
        let lnkdir = project_root.join(".vagga/.lnk");
        match readlink(&lnkdir) {
            Ok(lnk) => {
                if let Some(name) = lnk.filename() {
                    let target = dir.join(name);
                    if Path::new(lnk.dirname()) != *dir {
                        return Err(concat!("You have set storage_dir to {}, ",
                            "but .vagga/.lnk points to {}. You probably need ",
                            "to run `ln -sfn {} .vagga/.lnk`").to_string());
                    }
                    if !lnkdir.exists() {
                        return Err(format!("Your .vagga/.lnk points to a \
                            non-existent directory. Presumably you deleted \
                            dir {}. Just remove .vagga/.lnk now.",
                            lnk.display()));
                    }
                    return Ok(target);
                } else {
                    return Err(format!(concat!("Bad link .vagga/.lnk: {}.",
                        " You are pobably need to remove it now"),
                        lnk.display()));
                }
            }
            Err(ref e) if e.kind == FileNotFound => {
                let target = try!(create_storage_dir(dir, project_root));
                try!(safe_ensure_dir(&target));
                try_str!(symlink(&target, &lnkdir));
                try_str!(symlink(project_root, &target.join(".lnk")));
                return Ok(target)
            }
            Err(ref e) => {
                return Err(format!("Can't read link .vagga/.lnk: {}", e));
            }
        };
    } else {
        return Ok(project_root.join(".vagga"));
    }
}

fn make_mountpoint(project_root: &Path) -> Result<(), String> {
    let vagga_dir = project_root.join(".vagga");
    try!(safe_ensure_dir(&vagga_dir));
    let mnt_dir = vagga_dir.join(".mnt");
    try!(safe_ensure_dir(&mnt_dir));
    return Ok(());
}

pub fn setup_base_filesystem(project_root: &Path, settings: &MergedSettings)
    -> Result<(), String>
{
    let mnt_dir = project_root.join(".vagga/.mnt");
    try!(make_mountpoint(project_root));
    try!(mount_tmpfs(&mnt_dir, "size=10m"));

    let proc_dir = mnt_dir.join("proc");
    try_str!(mkdir(&proc_dir, ALL_PERMISSIONS));
    try!(mount_pseudo(&proc_dir, "proc", "", false));

    let dev_dir = mnt_dir.join("dev");
    try_str!(mkdir(&dev_dir, ALL_PERMISSIONS));
    try!(bind_mount(&Path::new("/dev"), &dev_dir));

    let sys_dir = mnt_dir.join("sys");
    try_str!(mkdir(&sys_dir, ALL_PERMISSIONS));
    try!(bind_mount(&Path::new("/sys"), &sys_dir));

    let vagga_dir = mnt_dir.join("vagga");
    try_str!(mkdir(&vagga_dir, ALL_PERMISSIONS));

    let bin_dir = vagga_dir.join("bin");
    try_str!(mkdir(&bin_dir, ALL_PERMISSIONS));
    try!(bind_mount(&self_exe_path().unwrap(), &bin_dir));
    try!(remount_ro(&bin_dir));

    let etc_dir = mnt_dir.join("etc");
    try_str!(mkdir(&etc_dir, ALL_PERMISSIONS));
    try!(copy(&Path::new("/etc/hosts"), &etc_dir.join("hosts"))
        .map_err(|e| format!("Error copying /etc/hosts: {}", e)));
    try!(copy(&Path::new("/etc/resolv.conf"), &etc_dir.join("resolv.conf"))
        .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));

    let roots_dir = vagga_dir.join("roots");
    try_str!(mkdir(&roots_dir, ALL_PERMISSIONS));
    let vagga_base = try!(vagga_base(project_root, settings));
    let local_roots = vagga_base.join(".roots");
    try!(safe_ensure_dir(&local_roots));
    try!(bind_mount(&local_roots, &roots_dir));

    let cache_dir = vagga_dir.join("cache");
    try_str!(mkdir(&cache_dir, ALL_PERMISSIONS));
    let locl_cache = try!(make_cache_dir(project_root, &vagga_base, settings));
    try!(bind_mount(&locl_cache, &cache_dir));

    let work_dir = mnt_dir.join("work");
    try_str!(mkdir(&work_dir, ALL_PERMISSIONS));
    try!(bind_mount(project_root, &work_dir));


    let old_root = vagga_dir.join("old_root");
    try_str!(mkdir(&old_root, ALL_PERMISSIONS));
    try!(change_root(&mnt_dir, &old_root));
    try!(unmount(&Path::new("/vagga/old_root")));

    return Ok(());
}

pub fn get_environment(container: &Container)
    -> Result<TreeMap<String, String>, String>
{
    let mut result = TreeMap::new();
    result.insert("TERM".to_string(),
                  getenv("TERM").unwrap_or("dumb".to_string()));
    result.insert("PATH".to_string(),
                  DEFAULT_PATH.to_string());
    if let Some(ref filename) = container.environ_file {
        let mut f = BufferedReader::new(try!(
                File::open(filename)
                .map_err(|e| format!("Error reading environment file {}: {}",
                    filename.display(), e))));
        for line_read in f.lines() {
            let line = try!(line_read
                .map_err(|e| format!("Error reading environment file {}: {}",
                    filename.display(), e)));
            let mut pair = line.as_slice().splitn(2, '=');
            let key = pair.next().unwrap();
            let mut value = try!(pair.next()
                .ok_or(format!("Error reading environment file {}: bad format",
                    filename.display())));
            if value.len() > 0 && value.starts_with("\"") {
                value = value[ 1 .. value.len()-2 ];
            }
            result.insert(key.to_string(), value.to_string());
        }
    }
    for (ref k, ref v) in container.environ.iter() {
        result.insert(k.to_string(), v.to_string());
    }
    return Ok(result);
}

pub fn setup_filesystem(container: &Container, container_ver: &str)
    -> Result<(), String>
{
    let root_path = Path::new("/");
    let tgtroot = Path::new("/vagga/root");
    if !tgtroot.exists() {
        try!(mkdir(&tgtroot, ALL_PERMISSIONS)
             .map_err(|x| format!("Error creating directory: {}", x)));
    }
    try!(bind_mount(&Path::new("/vagga/roots")
                     .join(container_ver).join("root"),
                    &tgtroot)
         .map_err(|e| format!("Error bind mount: {}", e)));
    try!(remount_ro(&tgtroot));
    try!(mount_system_dirs()
        .map_err(|e| format!("Error mounting system dirs: {}", e)));
    for (path, vol) in container.volumes.iter() {
        match vol {
            &Tmpfs(params) => {
                try!(mount_tmpfs(&tgtroot
                    .join(path.path_relative_from(&root_path).unwrap()),
                    format!("size={:u},mode=0{:o}", params.size, params.mode)
                    .as_slice()));
            }
        }
    }
    try!(change_root(&tgtroot, &tgtroot.join("tmp"))
         .map_err(|e| format!("Error changing root: {}", e)));
    try!(unmount(&Path::new("/work/.vagga/.mnt"))
         .map_err(|e| format!("Error unmounting `.vagga/.mnt`: {}", e)));
    try!(unmount(&Path::new("/tmp"))
         .map_err(|e| format!("Error unmounting `/tmp`: {}", e)));
    Ok(())
}
