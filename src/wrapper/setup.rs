use std::collections::BTreeMap;
use std::env;
use std::env::{current_exe};
use std::io::{BufRead, BufReader, ErrorKind};
use std::fs::{copy, read_link, hard_link, set_permissions, Permissions};
use std::fs::{remove_dir_all, read_dir, symlink_metadata};
use std::fs::File;
use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use libc::pid_t;

use super::super::config::Container;
use super::super::config::containers::Volume::{Tmpfs, VaggaBin, BindRW};
use super::super::container::root::{change_root};
use super::super::container::mount::{bind_mount, unmount, mount_system_dirs, remount_ro};
use super::super::container::mount::{mount_tmpfs, mount_pseudo, mount_proc};
use super::settings::{MergedSettings};
use process_util::DEFAULT_PATH;
use file_util::create_dir;
use path_util::{ToRelative, PathExt};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteMode {
    ReadOnly,
    TransientHardlinkCopy(pid_t),
}

fn create_storage_dir(storage_dir: &Path, project_root: &Path)
    -> Result<PathBuf, String>
{
    let name = match project_root.file_name().and_then(|x| x.to_str()) {
        Some(name) => name,
        None => return Err(format!(
            "Project dir `{}` is either root or has bad characters",
            project_root.display())),
    };
    let path = storage_dir.join(name);
    if !path.exists() {
        return Ok(path);
    }
    for i in 1..101 {
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
    -> Result<PathBuf, String>
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
    match symlink_metadata(dir) {
        Ok(ref stat) if stat.file_type().is_symlink() => {
            return Err(format!(concat!("The `{0}` dir can't be a symlink. ",
                               "Please run `unlink {0}`"), dir.display()));
        }
        Ok(ref stat) if stat.file_type().is_dir() => {
            // ok
        }
        Ok(_) => {
            return Err(format!(concat!("The `{0}` must be a directory. ",
                               "Please run `unlink {0}`"), dir.display()));
        }
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            try_msg!(create_dir(dir, false),
                "Can't create {dir:?}: {err}", dir=dir);
        }
        Err(ref e) => {
            return Err(format!("Can't stat `{}`: {}", dir.display(), e));
        }
    }
    return Ok(());
}

fn _vagga_base(project_root: &Path, settings: &MergedSettings)
    -> Result<Result<PathBuf, (PathBuf, PathBuf)>, String>
{
    if let Some(ref dir) = settings.storage_dir {
        let lnkdir = project_root.join(".vagga/.lnk");
        match read_link(&lnkdir) {
            Ok(lnk) => {
                if let Some(name) = lnk.file_name() {
                    let target = dir.join(name);
                    if lnk.parent() != Some(&*dir) {
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
                    return Ok(Ok(target));
                } else {
                    return Err(format!(concat!("Bad link .vagga/.lnk: {}.",
                        " You are pobably need to remove it now"),
                        lnk.display()));
                }
            }
            Err(ref e) if e.kind() == ErrorKind::NotFound => {
                return Ok(Err((lnkdir, dir.clone())));
            }
            Err(ref e) => {
                return Err(format!("Can't read link .vagga/.lnk: {}", e));
            }
        };
    } else {
        return Ok(Ok(project_root.join(".vagga")));
    }
}

pub fn get_vagga_base(project_root: &Path, settings: &MergedSettings)
    -> Result<Option<PathBuf>, String>
{
    return _vagga_base(project_root, settings).map(|x| x.ok());
}

fn vagga_base(project_root: &Path, settings: &MergedSettings)
    -> Result<PathBuf, String>
{
    match _vagga_base(project_root, settings) {
        Ok(Err((lnkdir, dir))) => {
            let target = try!(create_storage_dir(&dir, project_root));
            try!(safe_ensure_dir(&target));
            try!(symlink(&target, &lnkdir)
                .map_err(|e| format!("Error symlinking storage: {}", e)));
            try!(symlink(project_root, &target.join(".lnk"))
                .map_err(|e| format!("Error symlinking storage: {}", e)));
            return Ok(target)
        }
        Ok(Ok(path)) => {
            return Ok(path);
        }
        Err(e) => {
            return Err(e);
        }
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
    try!(mount_tmpfs(&mnt_dir, "size=100m"));

    let proc_dir = mnt_dir.join("proc");
    try_msg!(create_dir(&proc_dir, false),
             "Error creating /proc: {err}");
    try!(mount_proc(&proc_dir));

    let dev_dir = mnt_dir.join("dev");
    try_msg!(create_dir(&dev_dir, false),
             "Error creating /dev: {err}");
    try!(bind_mount(&Path::new("/dev"), &dev_dir));

    let sys_dir = mnt_dir.join("sys");
    try_msg!(create_dir(&sys_dir, false),
             "Error creating /sys: {err}");
    try!(bind_mount(&Path::new("/sys"), &sys_dir));

    let vagga_dir = mnt_dir.join("vagga");
    try_msg!(create_dir(&vagga_dir, false),
             "Error creating /vagga: {err}");

    let bin_dir = vagga_dir.join("bin");
    try_msg!(create_dir(&bin_dir, false),
             "Error creating /vagga/bin: {err}");
    try!(bind_mount(&current_exe().unwrap().parent().unwrap(), &bin_dir));
    try!(remount_ro(&bin_dir));

    let etc_dir = mnt_dir.join("etc");
    try_msg!(create_dir(&etc_dir, false),
             "Error creating /etc: {err}");
    try!(copy(&Path::new("/etc/hosts"), &etc_dir.join("hosts"))
        .map_err(|e| format!("Error copying /etc/hosts: {}", e)));
    try!(copy(&Path::new("/etc/resolv.conf"), &etc_dir.join("resolv.conf"))
        .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));

    let local_base = vagga_dir.join("base");
    try!(safe_ensure_dir(&local_base));
    let vagga_base = try!(vagga_base(project_root, settings));

    try!(bind_mount(&vagga_base, &local_base));
    try!(safe_ensure_dir(&local_base.join(".roots")));
    try!(safe_ensure_dir(&local_base.join(".transient")));

    let cache_dir = vagga_dir.join("cache");
    try_msg!(create_dir(&cache_dir, false),
        "Error creating /vagga/cache: {err}");
    let locl_cache = try!(make_cache_dir(project_root, &vagga_base, settings));
    try!(bind_mount(&locl_cache, &cache_dir));

    if let Ok(nsdir) = env::var("VAGGA_NAMESPACE_DIR") {
        let newns_dir = vagga_dir.join("namespaces");
        try_msg!(create_dir(&newns_dir, true),
            "Error creating directory for namespaces: {err}");
        try!(bind_mount(&Path::new(&nsdir), &newns_dir)
             .map_err(|e| format!("Error mounting directory \
                with namespaces: {}", e)));
    }

    let work_dir = mnt_dir.join("work");
    try_msg!(create_dir(&work_dir, false),
        "Error creating /work: {err}");
    try!(bind_mount(project_root, &work_dir));


    let old_root = vagga_dir.join("old_root");
    try_msg!(create_dir(&old_root, false),
             "Error creating /vagga/old_root: {err}");
    try!(change_root(&mnt_dir, &old_root));
    try!(unmount(&Path::new("/vagga/old_root")));

    Ok(())
}

pub fn get_environment(container: &Container)
    -> Result<BTreeMap<String, String>, String>
{
    let mut result = BTreeMap::new();
    result.insert("TERM".to_string(),
                  env::var("TERM").unwrap_or("dumb".to_string()));
    result.insert("PATH".to_string(),
                  DEFAULT_PATH.to_string());
    for (k, v) in env::vars() {
        if k.starts_with("VAGGAENV_") {
            result.insert(k[9..].to_string(), v);
        }
    }
    if let Some(ref filename) = container.environ_file {
        let f = BufReader::new(try!(
                File::open(filename)
                .map_err(|e| format!("Error reading environment file {}: {}",
                    filename.display(), e))));
        for line_read in f.lines() {
            let line = try!(line_read
                .map_err(|e| format!("Error reading environment file {}: {}",
                    filename.display(), e)));
            let mut pair = line[..].splitn(2, '=');
            let key = pair.next().unwrap();
            let mut value = try!(pair.next()
                .ok_or(format!("Error reading environment file {}: bad format",
                    filename.display())));
            if value.len() > 0 && value.starts_with("\"") {
                value = value.trim_matches('"');
            }
            result.insert(key.to_string(), value.to_string());
        }
    }
    for (ref k, ref v) in container.environ.iter() {
        result.insert(k.to_string(), v.to_string());
    }
    return Ok(result);
}

fn hardlink_dir(old: &Path, new: &Path) -> Result<(), String> {
    for entry in try_msg!(read_dir(old), "Can't open dir {d:?}: {err}", d=old) {
        let entry = try_msg!(entry, "Can't read dir entry {d:?}: {err}", d=old);
        let stat = try_msg!(symlink_metadata(entry.path()),
            "Can't stat file {path:?}: {err}", path=entry.path());
        let nitem = new.join(entry.file_name());
        let typ = try_msg!(entry.file_type(),
            "Can't stat {f:?}: {err}", f=entry.path());
        if typ.is_file() {
            try!(hard_link(&entry.path(), &nitem)
                .map_err(|e| format!("Can't hard-link file: {}", e)));
        } else if typ.is_dir() {
            try_msg!(create_dir(&nitem, false),
                     "Can't create dir: {err}");
            try!(set_permissions(&nitem, Permissions::from_mode(stat.mode()))
                .map_err(|e| format!("Can't chmod: {}", e)));
            try!(hardlink_dir(&entry.path(), &nitem));
        } else if typ.is_symlink() {
            let lnk = try!(read_link(&entry.path())
                .map_err(|e| format!("Can't readlink: {}", e)));
            try!(symlink(&lnk, &nitem)
                .map_err(|e| format!("Can't symlink: {}", e)));
        } else {
            warn!("Unknown file type for {:?}", entry.path());
        }
    }
    Ok(())
}

pub fn setup_filesystem(container: &Container, write_mode: WriteMode,
    container_ver: &str)
    -> Result<(), String>
{
    let tgtroot = Path::new("/vagga/root");
    if !tgtroot.exists() {
        try_msg!(create_dir(&tgtroot, false),
                 "Can't create rootfs mountpoint: {err}");
    }
    match write_mode {
        WriteMode::ReadOnly => {
            let nroot = Path::new("/vagga/base/.roots")
                .join(container_ver).join("root");
            try!(bind_mount(&nroot, &tgtroot)
                 .map_err(|e| format!("Error bind mount: {}", e)));
        }
        WriteMode::TransientHardlinkCopy(pid) => {
            let oldpath = Path::new("/vagga/base/.roots")
                .join(container_ver).join("root");
            let newpath = Path::new("/vagga/base/.transient")
                .join(format!("{}.{}", container_ver, pid));
            if newpath.exists() {
                try_msg!(remove_dir_all(&newpath),
                        "Error removing dir: {err}");
            }
            try_msg!(create_dir(&newpath, false),
                     "Error creating directory: {err}");
            try!(hardlink_dir(&oldpath, &newpath));
            try!(bind_mount(&newpath, &tgtroot)
                 .map_err(|e| format!("Error bind mount: {}", e)));
        }
    };

    try!(mount_system_dirs()
        .map_err(|e| format!("Error mounting system dirs: {}", e)));

    if let None = container.volumes.get(&PathBuf::from("/tmp")) {
        try!(mount_tmpfs(&tgtroot.join("tmp"), "size=100m,mode=01777"));
    }

    for (path, vol) in container.volumes.iter() {
        let dest = tgtroot.join(path.rel());
        match vol {
            &Tmpfs(ref params) => {
                try!(mount_tmpfs(&dest,
                    &format!("size={},mode=0{:o}", params.size, params.mode)));
            }
            &VaggaBin => {
                try!(bind_mount(&Path::new("/vagga/bin"), &dest));
            }
            &BindRW(ref bindpath) => {
                let src = tgtroot.join(bindpath.rel());
                try!(bind_mount(&src, &dest));
            }
        }
    }
    if let Ok(_) = env::var("VAGGA_NAMESPACE_DIR") {
        let newns_dir = tgtroot.join("tmp/vagga/namespaces");
        try_msg!(create_dir(&newns_dir, true),
            "Error creating directory for namespaces: {err}");
        try!(bind_mount(&Path::new("/vagga/namespaces"), &newns_dir)
             .map_err(|e| format!("Error mounting directory \
                with namespaces: {}", e)));
    }

    if let Some(ref path) = container.resolv_conf_path {
        let path = tgtroot.join(path.rel());
        try!(copy(&Path::new("/etc/resolv.conf"), &path)
            .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));
    }

    //  Currently we need the root to be writeable for putting resolv.conf
    //  It's a bit ugly but bearable for development environments.
    //  Eventually we'll find a better way
    if write_mode == WriteMode::ReadOnly {
        try!(remount_ro(&tgtroot));
    }

    try!(change_root(&tgtroot, &tgtroot.join("tmp"))
         .map_err(|e| format!("Error changing root: {}", e)));
    try!(unmount(&Path::new("/work/.vagga/.mnt"))
         .map_err(|e| format!("Error unmounting `.vagga/.mnt`: {}", e)));
    try!(unmount(&Path::new("/tmp"))
         .map_err(|e| format!("Error unmounting `/tmp`: {}", e)));
    Ok(())
}

